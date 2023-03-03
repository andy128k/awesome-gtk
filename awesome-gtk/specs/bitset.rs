use awesome_gtk::prelude::*;

pub fn test_bitset_iter() {
    let empty = gtk::Bitset::new_empty();
    let range = gtk::Bitset::new_range(12, 3);
    let odds = gtk::Bitset::new_empty();
    odds.add(3);
    odds.add(7);
    odds.add(5);

    assert_eq!(empty.iter_asc().count(), 0);
    assert_eq!(range.iter_asc().collect::<Vec<u32>>(), vec![12, 13, 14]);
    assert_eq!(odds.iter_asc().collect::<Vec<u32>>(), vec![3, 5, 7]);

    assert_eq!(empty.iter_desc().count(), 0);
    assert_eq!(range.iter_desc().collect::<Vec<u32>>(), vec![14, 13, 12]);
    assert_eq!(odds.iter_desc().collect::<Vec<u32>>(), vec![7, 5, 3]);
}
