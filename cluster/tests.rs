/// Imports `ink_lang` so we can use `#[ink::test]`.
use ink_lang as ink;

/// Imports all the definitions from the outer scope so we can use them here.
use super::cluster::*;

/// We test if the default constructor does its job.
#[ink::test]
fn cluster_works() {
    const PRICE: u128 = 10;
    const LOCATION: &str = "https://somewhere/{RESOURCE_ID}";

    // As Cluster Owner.
    let mut cluster = Cluster::default();

    cluster.set_price(PRICE).unwrap();
    cluster.set_location(LOCATION.to_string()).unwrap();

    // As App Developer.
    let price = cluster.get_price().unwrap();
    assert_eq!(price, PRICE);

    let res_id1 = cluster.create_resource().unwrap();
    let res_id2 = cluster.create_resource().unwrap();
    assert_ne!(res_id1, res_id2);

    // As App.
    let resource_location = cluster.get_location().unwrap();
    assert_eq!(resource_location, LOCATION);
}
