use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use std::collections::BTreeMap;
use syn::{
    meta::ParseNestedMeta, parse, spanned::Spanned, Attribute, Error, FnArg, ImplItem, ImplItemFn,
    ItemImpl, Lit, LitStr, PatType, Result, Signature, Type,
};

#[derive(Default)]
pub struct ActionImplAttributes {
    register_fn: Option<LitStr>,
}

impl ActionImplAttributes {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("register_fn") {
            self.register_fn = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported actions property"))
        }
    }
}

#[derive(Default)]
struct ActionAttributes {
    name: Option<String>,
    stateful: bool,
    initial_state: Option<Lit>,
    change_state: bool,
    no_parameter: bool,
}

impl ActionAttributes {
    fn parse(attributes: &[Attribute]) -> Result<ActionAttributes> {
        let mut action_attributes = ActionAttributes::default();
        for attr in attributes {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    action_attributes.name = Some(meta.value()?.parse::<LitStr>()?.value());
                    Ok(())
                } else if meta.path.is_ident("stateful") {
                    action_attributes.stateful = true;
                    Ok(())
                } else if meta.path.is_ident("change_state") {
                    action_attributes.change_state = true;
                    Ok(())
                } else if meta.path.is_ident("no_parameter") {
                    action_attributes.no_parameter = true;
                    Ok(())
                } else if meta.path.is_ident("initial_state") {
                    action_attributes.initial_state = Some(meta.value()?.parse::<Lit>()?);
                    Ok(())
                } else {
                    Err(meta.error("unrecognized option"))
                }
            })?;
        }
        Ok(action_attributes)
    }
}

struct ActionHandlerInfo {
    attrs: ActionAttributes,
    sig: Signature,
}

impl ActionHandlerInfo {
    fn action_name(&self) -> String {
        if let Some(ref name) = self.attrs.name {
            name.clone()
        } else {
            self.sig.ident.to_string()
        }
    }
}

fn expect_one<T>(mut vec: Vec<T>, make_item_error: impl Fn(&T) -> Error) -> Result<Option<T>> {
    match vec.len() {
        0 => Ok(None),
        1 => Ok(Some(vec.remove(0))),
        _ => {
            let mut error_acc = None;
            for item in &vec[1..] {
                let item_error = (make_item_error)(item);
                combine_errors(&mut error_acc, item_error);
            }
            Err(error_acc.unwrap())
        }
    }
}

struct ActivateHandler {
    sig: Signature,
    state_arg_index: Option<usize>,
    parameter_arg_index: Option<usize>,
}

impl ActivateHandler {
    fn try_from_signature(sig: Signature, stateful: bool) -> Result<Self> {
        if !is_assoc(&sig) {
            return Err(Error::new(
                sig.span(),
                "Unsupported signature of method. Only associated methods are supported.",
            ));
        }

        let state_arg_index;
        let parameter_arg_index;
        match sig.inputs.len() {
            1 => {
                state_arg_index = None;
                parameter_arg_index = None;
            },
            2 if stateful => {
                state_arg_index = Some(1);
                parameter_arg_index = None;
            },
            2 => {
                state_arg_index = None;
                parameter_arg_index = Some(1);
            },
            3 if stateful => {
                state_arg_index = Some(1);
                parameter_arg_index = Some(2);
            },
            n => return Err(Error::new(
                sig.span(),
                format!("Unsupported signature of method. It has {} parameters but only 0, 1, or 2 are supported.", n - 1)
            )),
        }

        Ok(Self {
            sig,
            state_arg_index,
            parameter_arg_index,
        })
    }

    fn state_arg(&self) -> Option<&FnArg> {
        self.state_arg_index.map(|index| &self.sig.inputs[index])
    }

    fn parameter_arg(&self) -> Option<&FnArg> {
        self.parameter_arg_index
            .map(|index| &self.sig.inputs[index])
    }

    fn state_type(&self) -> Result<Option<&Type>> {
        self.state_arg().map(argument_type).transpose()
    }

    fn parameter_type(&self) -> Result<Option<&Type>> {
        self.parameter_arg().map(argument_type).transpose()
    }
}

struct ChangeStateHandler {
    sig: Signature,
    no_parameter: bool,
}

impl ChangeStateHandler {
    fn try_from_signature(sig: Signature, no_parameter: bool) -> Result<Self> {
        if !is_assoc(&sig) {
            return Err(Error::new(
                sig.span(),
                "Unsupported signature of a method. Only associated methods are supported.",
            ));
        }
        if sig.inputs.len() != 2 {
            return Err(Error::new(
                sig.span(),
                format!(
                    "Unsupported signature of a method. It has {} parameters but only 1 is supported.",
                    sig.inputs.len() - 1
                ),
            ));
        }
        Ok(Self { sig, no_parameter })
    }

    fn state_arg(&self) -> &FnArg {
        &self.sig.inputs[1]
    }

    fn state_type(&self) -> Result<&Type> {
        argument_type(self.state_arg())
    }
}

struct ActionInfo {
    name: String,
    initial_state: Option<Lit>,
    activate_handler: Option<ActivateHandler>,
    change_state_handler: Option<ChangeStateHandler>,
}

impl ActionInfo {
    fn try_from_handlers(name: String, handlers: Vec<ActionHandlerInfo>) -> Result<Self> {
        let (change_state_handlers, activate_handlers): (Vec<_>, Vec<_>) = handlers
            .into_iter()
            .partition(|h| h.attrs.change_state.into());

        let mut activate_handler = expect_one(activate_handlers, |handler| {
            Error::new(
                handler.sig.span(),
                format!("Duplicated `activate` handler for an action {}.", name),
            )
        })?;

        let mut change_state_handler = expect_one(change_state_handlers, |handler| {
            Error::new(
                handler.sig.span(),
                format!("Duplicated `change-state` handler for an action {}.", name),
            )
        })?;

        let initial_state = match (
            activate_handler
                .as_mut()
                .and_then(|h| h.attrs.initial_state.take()),
            change_state_handler
                .as_mut()
                .and_then(|h| h.attrs.initial_state.take()),
        ) {
            (None, None) => None,
            (v @ Some(..), None) | (None, v @ Some(..)) => v,
            (Some(..), Some(value)) => {
                return Err(Error::new(
                    value.span(),
                    format!("Duplicated initial state for an action {}.", name),
                ))
            }
        };

        let stateful = activate_handler
            .as_ref()
            .map_or(false, |h| h.attrs.stateful.into())
            || initial_state.is_some()
            || change_state_handler.is_some();

        let activate_handler = activate_handler
            .map(|h| ActivateHandler::try_from_signature(h.sig, stateful))
            .transpose()?;

        let no_parameter = change_state_handler
            .as_ref()
            .map_or(false, |h| h.attrs.no_parameter.into());
        let change_state_handler = change_state_handler
            .map(|h| ChangeStateHandler::try_from_signature(h.sig, no_parameter))
            .transpose()?;

        Ok(Self {
            name,
            initial_state,
            activate_handler,
            change_state_handler,
        })
    }

    fn state_type(&self) -> Result<Option<&Type>> {
        if let Some(ref h) = self.change_state_handler {
            h.state_type().map(Some)
        } else if let Some(ref h) = self.activate_handler {
            h.state_type()
        } else {
            Ok(None)
        }
    }

    fn parameter_type(&self) -> Result<Option<&Type>> {
        match (&self.activate_handler, &self.change_state_handler) {
            (Some(ref handler), _) => handler.parameter_type(),
            (None, Some(ref handler)) if handler.no_parameter => Ok(None),
            (None, Some(ref handler)) => handler.state_type().map(Some),
            (None, None) => Ok(None),
        }
    }
}

fn get_parameter(info: &ActionInfo, arg: &FnArg) -> Result<TokenStream> {
    let action_name = &info.name;
    let parameter_type = argument_type(arg)?;
    Ok(quote_spanned! { arg.span() =>
        #[allow(clippy::redundant_closure)]
        match parameter.as_ref().and_then(|variant| <#parameter_type as glib::variant::FromVariant>::from_variant(variant)) {
            Some(parameter) => parameter,
            None => {
                glib::g_critical!("actions", "Action {} expects a parameter of type {} but received `{:?}`.", #action_name, stringify!(#parameter_type), parameter);
                return;
            },
        }
    })
}

fn get_state(info: &ActionInfo, arg: &FnArg) -> Result<TokenStream> {
    let action_name = &info.name;
    let state_type = argument_type(arg)?;
    Ok(quote_spanned! { arg.span() =>
        match action.state().and_then(|variant| variant.get::<#state_type>()) {
            Some(value) => value,
            None => {
                glib::g_critical!("actions", "Action {} expects a state of type {} but has `{:?}`.", #action_name, stringify!(#state_type), action.state());
                return;
            }
        }
    })
}

fn change_state(span: Span, expression: &TokenStream, state_type: &Type) -> TokenStream {
    quote_spanned! { span => {
        #[allow(clippy::useless_conversion)]
        let new_state_opt: Option<#state_type> = (#expression).into();
        if let Some(ref new_state) = new_state_opt {
            action.change_state(&<#state_type as glib::variant::ToVariant>::to_variant(new_state));
        }
    }}
}

fn add_comma(expression: TokenStream) -> TokenStream {
    quote! { #expression , }
}

fn maybe_await(is_async: bool) -> TokenStream {
    if is_async {
        quote! { .await }
    } else {
        quote! {}
    }
}

fn generate_activate_handler(info: &ActionInfo, handler: &ActivateHandler) -> Result<TokenStream> {
    let state_arg = handler
        .state_arg()
        .map(|arg| get_state(info, arg))
        .transpose()?
        .map(add_comma);

    let parameter_arg = handler
        .parameter_arg()
        .map(|arg| get_parameter(info, arg))
        .transpose()?
        .map(add_comma);

    let method = &handler.sig.ident;
    let is_async = handler.sig.asyncness.is_some();
    let maybe_await = maybe_await(is_async);
    let mut invoke = quote_spanned! { handler.sig.span() =>
        this.#method(#state_arg #parameter_arg) #maybe_await
    };
    if let Some(state_type) = handler.state_type()? {
        invoke = change_state(handler.sig.output.span(), &invoke, state_type);
    }

    if is_async {
        invoke = quote! {
            let action = action.clone();
            let parameter: Option<glib::variant::Variant> = parameter.cloned();
            glib::MainContext::default().spawn_local(async move { #invoke });
        };
    }

    let handler = quote_spanned! { handler.sig.span() =>
        #[allow(unused_variables, unused_braces)]
        action.connect_activate(glib::clone!(
            #[weak(rename_to = this)] self,
            move |action, parameter| { #invoke },
        ));
    };
    Ok(handler)
}

fn generate_change_state_handler(
    info: &ActionInfo,
    handler: &ChangeStateHandler,
) -> Result<TokenStream> {
    let action_name = &info.name;
    let method = &handler.sig.ident;
    let is_async = handler.sig.asyncness.is_some();
    let maybe_await = maybe_await(is_async);
    let state_type = handler.state_type()?;
    let mut invoke = quote_spanned! { handler.sig.span() =>
        let new_state: #state_type = match new_state_opt.and_then(|state| state.get()) {
            Some(value) => value,
            None => {
                glib::g_critical!("actions", "State of type {} is expected in action {} but it is None.", stringify!(#state_type), #action_name);
                return;
            }
        };
        let result: Option<#state_type> = this.#method(new_state) #maybe_await .into();
        if let Some(ref new_state) = result {
            action.set_state(&<#state_type as glib::variant::ToVariant>::to_variant(new_state));
        }
    };
    if is_async {
        invoke = quote! {
            let action = action.clone();
            let new_state_opt: Option<glib::variant::Variant> = new_state_opt.cloned();
            glib::MainContext::default().spawn_local(async move { #invoke });
        };
    }
    Ok(quote_spanned! { handler.sig.span() =>
        #[allow(unused_variables, unused_braces)]
        action.connect_change_state(glib::clone!(
            #[weak(rename_to = this)] self,
            move |action, new_state_opt| { #invoke },
        ));
    })
}

fn generate_action(info: &ActionInfo) -> Result<TokenStream> {
    let action_name = &info.name;

    let parameter = if let Some(parameter_type) = info.parameter_type()? {
        quote_spanned! { parameter_type.span() =>
            Some(<#parameter_type as glib::variant::StaticVariantType>::static_variant_type().as_ref())
        }
    } else {
        quote! {
            None
        }
    };

    let create = if let Some(state_type) = info.state_type()? {
        let initial_state_expr = if let Some(ref value) = info.initial_state {
            quote_spanned! { value.span() =>
                <#state_type as std::convert::From<_>>::from(#value)
            }
        } else {
            quote_spanned! { state_type.span() =>
                <#state_type as std::default::Default>::default()
            }
        };
        quote! {
            let initial_state: #state_type = #initial_state_expr;
            let initial_state_variant = <#state_type as glib::variant::ToVariant>::to_variant(&initial_state);
            let action = gio::SimpleAction::new_stateful(#action_name, #parameter, &initial_state_variant);
        }
    } else {
        quote! {
            let action = gio::SimpleAction::new(#action_name, #parameter);
        }
    };

    let activate_handler = info
        .activate_handler
        .as_ref()
        .map(|handler| generate_activate_handler(info, handler))
        .transpose()?;

    let change_state_handler = info
        .change_state_handler
        .as_ref()
        .map(|handler| generate_change_state_handler(info, handler))
        .transpose()?;

    let action = quote! {
        {
            #create
            #activate_handler
            #change_state_handler
            action
        }
    };
    Ok(action)
}

fn combine_errors(error_acc: &mut Option<Error>, error: Error) {
    match error_acc {
        Some(ref mut error_acc) => {
            error_acc.combine(error);
        }
        None => {
            error_acc.replace(error);
        }
    }
}

fn is_assoc(sig: &Signature) -> bool {
    sig.inputs
        .first()
        .map_or(false, |arg| matches!(arg, FnArg::Receiver(..)))
}

fn argument_type(arg: &FnArg) -> Result<&Type> {
    match arg {
        FnArg::Typed(PatType { ty, .. }) => Ok(&*ty),
        _ => Err(Error::new(
            arg.span(),
            "Cannot extract type of an argument.",
        )),
    }
}

fn generate_register_method(
    attrs: &ActionImplAttributes,
    actions: &[TokenStream],
) -> Result<ImplItemFn> {
    let register_fn = match attrs.register_fn {
        Some(ref name) => format_ident!("{}", name.value()),
        None => format_ident!("register_actions"),
    };
    let register_method = quote! {
        #[allow(clippy)]
        fn #register_fn<AM: glib::object::IsA<gio::ActionMap>>(&self, map: &AM) {
            #(
                map.add_action(& #actions );
            )*
        }
    };
    parse(register_method.into())
}

pub fn actions(attrs: ActionImplAttributes, mut input: ItemImpl) -> Result<TokenStream> {
    let mut action_handlers: BTreeMap<String, Vec<ActionHandlerInfo>> = BTreeMap::new();
    for item in input.items.iter_mut() {
        if let ImplItem::Fn(method) = item {
            let attributes =
                extract_from_vec(&mut method.attrs, |attr| attr.path().is_ident("action"));
            let info = ActionHandlerInfo {
                attrs: ActionAttributes::parse(&attributes)?,
                sig: method.sig.clone(),
            };
            action_handlers
                .entry(info.action_name())
                .or_default()
                .push(info);
        }
    }

    let action_infos: Vec<ActionInfo> = action_handlers
        .into_iter()
        .map(|(name, handlers)| ActionInfo::try_from_handlers(name, handlers))
        .collect::<Result<_>>()?;

    let action_definitions: Vec<TokenStream> = action_infos
        .iter()
        .map(generate_action)
        .collect::<Result<Vec<_>>>()?;

    let register_method = generate_register_method(&attrs, &action_definitions)?;
    input.items.push(ImplItem::Fn(register_method));

    Ok(quote!(#input).into())
}

// TODO: Replace this by Vec::drain_filter as soon as it is stabilized.
fn extract_from_vec<T>(vec: &mut Vec<T>, predicate: impl Fn(&T) -> bool) -> Vec<T> {
    let mut i = 0;
    let mut result: Vec<T> = Vec::new();
    while i != vec.len() {
        if (predicate)(&vec[i]) {
            let item = vec.remove(i);
            result.push(item);
        } else {
            i += 1;
        }
    }
    result
}
