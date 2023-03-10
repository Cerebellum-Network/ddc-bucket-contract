// use ink_lang as ink;

// use crate::ddc_bucket::*;

// use super::env_utils::*;

// #[ink::test]
// fn admin_init_works() {
//     let contract = setup();

//     // The deployer is SuperAdmin.
//     assert!(contract.has_permission(admin_id(), Permission::SuperAdmin));
//     assert!(!contract.has_permission(not_admin_id(), Permission::SuperAdmin));

//     // The SuperAdmin has all other permissions, too.
//     assert!(contract.has_permission(admin_id(), Permission::SetExchangeRate));
//     assert!(!contract.has_permission(not_admin_id(), Permission::SetExchangeRate));
// }

// #[ink::test]
// fn admin_withdraw_works() {
//     let mut contract = setup();

//     push_caller(admin_id());
//     contract.admin_withdraw(9);
//     pop_caller();

//     assert_eq!(balance_of(contract_id()), 1);
// }

// #[ink::test]
// #[should_panic]
// fn admin_withdraw_only_admin() {
//     let mut contract = setup();

//     push_caller(not_admin_id());
//     contract.admin_withdraw(9); // panic.
//     pop_caller();
// }

// #[ink::test]
// fn admin_grant_works() {
//     let mut contract = setup();
//     let permission = Permission::SuperAdmin;

//     push_caller_value(admin_id(), CONTRACT_FEE_LIMIT);
//     contract.admin_grant_permission(not_admin_id(), permission);
//     pop_caller();

//     // Check the last event.
//     let ev = get_events().pop().unwrap();
//     assert!(matches!(ev, Event::GrantPermission(ev) if ev ==
//         GrantPermission { account_id: not_admin_id(), permission }));

//     assert!(contract.has_permission(not_admin_id(), permission));

//     push_caller(not_admin_id());
//     contract.admin_withdraw(9);
//     pop_caller();
// }

// #[ink::test]
// #[should_panic]
// fn admin_grant_only_admin() {
//     let mut contract = DdcBucket::new();

//     push_caller_value(not_admin_id(), CONTRACT_FEE_LIMIT);
//     contract.admin_grant_permission(get_accounts().charlie, Permission::SuperAdmin); // panic.
//     pop_caller();
// }

// #[ink::test]
// #[should_panic]
// fn admin_revoke_works() {
//     let mut contract = setup();
//     let permission = Permission::SuperAdmin;

//     // Revoke the permission.
//     push_caller(admin_id());
//     contract.admin_revoke_permission(admin_id(), permission);
//     pop_caller();

//     // Check the last event.
//     let ev = get_events().pop().unwrap();
//     assert!(matches!(ev, Event::RevokePermission(ev) if ev ==
//         RevokePermission { account_id: not_admin_id(), permission }));

//     assert!(!contract.has_permission(admin_id(), permission));

//     // Cannot withdraw because no more permission.
//     push_caller(admin_id());
//     contract.admin_withdraw(9); // panic.
//     pop_caller();
// }

// #[ink::test]
// #[should_panic]
// fn admin_revoke_only_admin() {
//     let mut contract = DdcBucket::new();

//     push_caller_value(not_admin_id(), CONTRACT_FEE_LIMIT);
//     contract.admin_revoke_permission(admin_id(), Permission::SuperAdmin); // panic.
//     pop_caller();
// }

// fn setup() -> DdcBucket {
//     let contract = DdcBucket::new();
//     set_balance(contract_id(), 10);
//     contract
// }

// fn admin_id() -> AccountId {
//     get_accounts().alice
// }

// fn not_admin_id() -> AccountId {
//     get_accounts().bob
// }
