use ink_env::AccountId;
use ink_lang as ink;

use super::cluster::*;

#[ink::test]
fn cluster_works() {
    const PRICE: u128 = 10;
    const LOCATION: &str = "https://somewhere/{RESOURCE_ID}";

    use ink_env::call::FromAccountId;
    //let pa = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>().expect("Cannot get contract id");
    let payments = payments::PaymentsRef::from_account_id(AccountId::default());


    // As Cluster Owner.
    let mut cluster = Cluster::new(payments);

    cluster.set_price(PRICE)?;
    cluster.set_location(LOCATION.to_string())?;

    // As App Developer.
    let price = cluster.get_price()?;
    assert_eq!(price, PRICE);

    let res_id1 = cluster.create_resource()?;
    let res_id2 = cluster.create_resource()?;
    assert_ne!(res_id1, res_id2);

    // As App.
    let resource_location = cluster.get_location()?;
    assert_eq!(resource_location, LOCATION);
}
