use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{Vector, UnorderedSet, LookupMap};
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise, PromiseResult};
use near_sdk::serde::Serialize;

// use rand::{Rng, SeedableRng};

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
pub struct Digital {
    pub owner: AccountId,
    pub digital: u64,
    pub level: u32,
}

impl Digital {
    pub fn new(digital: u64, owner: AccountId) -> Self {
        Self {
            owner: owner,
            digital: digital,
            level: 1,
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct DigitalCenter {
    pub account_indices: LookupMap<AccountId, UnorderedSet<u64>>,
    pub digitals: Vector<Digital>,
    pub next: u64
}

#[near_bindgen]
impl DigitalCenter {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            account_indices: LookupMap::new(b"a".to_vec()),
            digitals: Vector::new(b"v".to_vec()),
            next: 0,
        }
    }

    pub fn next_digital(&self) -> u64{
        self.next
    }

    pub fn add_first(&mut self) -> String{
        let account_id= env::signer_account_id();
        let mut digital_set = self.account_indices.get(&account_id).unwrap_or(UnorderedSet::new(account_id.clone().into_bytes()));
        if digital_set.len() > 0 {
            return "You already have more than one digital".to_string();
        }
        digital_set.insert(&self.next);
        self.account_indices.insert(&account_id, &digital_set);
        self.digitals.push(&Digital::new(self.next, account_id));
        self.next += 1;
        "success".to_string()
    }

    pub fn pk(&mut self, own_digital: u64, target_digital: u64) -> String{
        let account_id= env::signer_account_id();
        if own_digital == target_digital {
            return "own_digital can not eq target_digital".to_string()
        }
        let mut own = match self.digitals.get(own_digital){
            Some(r) => r,
            None => return "Invalid own_digital".to_string()
        };
        let mut target = match self.digitals.get(target_digital){
            Some(r) => r,
            None => return "Invalid target_digital".to_string()
        };

        let mut own_digital_set = self.account_indices.get(&account_id).unwrap();
        if !own_digital_set.contains(&own_digital){
            return "you are not own_digital owner".to_string();
        }

        if own_digital_set.contains(&target_digital){
            return "can not pk your own digital".to_string();
        }

        let index = if env::random_seed().len() as u64 > env::block_timestamp() % 10  { env::block_timestamp() % 10 } else { env::random_seed().len() as u64 - 1};
        let rand_own = *env::random_seed().get(index as usize).unwrap() ;
        let rand_target  = *env::random_seed().get((if index - 1 > 0 {index - 1} else {index}) as usize).unwrap();

        let target_owner = target.owner.clone();
        let mut target_digital_set = self.account_indices.get(&target_owner).unwrap();

        let mut result = String::new();
        if (rand_own as u32 * (own.level + 100) / 100) > (rand_target as u32  * (target.level + 100) / 100){
            if 0 == target.level - 1 {
                target.owner = account_id.clone();
                
                result.push_str("you are win, get target digital!");
                target_digital_set.remove(&target_digital);
                own_digital_set.insert(&target_digital);
            } else {
                target.level -= 1;
                result.push_str("you are win, target digital level minus 1!");
            }
            own.level += 1;
            
        }else {
            if 0 == own.level - 1 {
                own.owner = target_owner.clone();
                result.push_str("you are lose, target get your digital!");
                own_digital_set.remove(&own_digital);
                target_digital_set.insert(&own_digital);
            } else {
                own.level -= 1;
                result.push_str("you are lose, your digital level minus 1!");
            }
            target.level += 1;
            
        }

        self.digitals.replace(own_digital, &own);
        self.digitals.replace(target_digital, &target);
            
        self.account_indices.insert(&account_id, &own_digital_set);
        self.account_indices.insert(&target_owner, &target_digital_set);
        result
    }

    pub fn get_digitals(&self, accountId: AccountId) -> Vec<u64> {
        let mut result = vec![];
        let set = match self.account_indices.get(&accountId){
            Some(r) => r,
            None => return result
        };

        set.iter().map(|item| {
            item
        }).for_each(|item|{
            result.push(item)
        }); 
        
        result
    }

    pub fn get_all_digitals(&self) -> Vec<Digital> {
        let mut result = vec![];
        self.digitals.iter().map(|item| {
            item
        }).for_each(|item|{
            result.push(item)
        }); 
        
        result
    }

    #[payable]
    pub fn levelup(&mut self, digital: u64) -> String {
        assert!(env::attached_deposit() >= 10, "Deposit is too low");
        let mut old = match self.digitals.get(digital){
            Some(r) => r,
            None => return "Invalid digital".to_string()
        };
        old.level += 1;
        self.digitals.replace(digital, &old);
        "levelup success".to_string()
    }
}


#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::{testing_env, MockedBlockchain, VMContext};

    pub fn get_context(accountId: AccountId, block_timestamp: u64) -> VMContext {
        VMContext {
            current_account_id: accountId.clone(),
            signer_account_id: accountId.clone(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: accountId,
            input: vec![],
            block_index: 1,
            block_timestamp,
            epoch_height: 1,
            account_balance: 10u128,//.pow(26)
            account_locked_balance: 0,
            storage_usage: 10u64.pow(6),
            attached_deposit: 0,
            prepaid_gas: 300 * 10u64.pow(12),
            random_seed: vec![0, 2, 1, 3, 4, 5, 6, 7],
            is_view: false,
            output_data_receivers: vec![],
        }
    }

    #[test]
    fn test_add_first_work() {
        let mut context = get_context("digital.test".to_string(), 3_600_000_000_000);
        testing_env!(context.clone());
        let mut contract = DigitalCenter::new();

        assert_eq!(contract.add_first(), "success".to_string());

        assert_eq!(contract.next_digital(), 1);
    }

    #[test]
    fn test_repeat_add_first() {
        let mut context = get_context("digital.test".to_string(), 3_600_000_000_000);
        testing_env!(context.clone());
        let mut contract = DigitalCenter::new();

        assert_eq!(contract.add_first(), "success".to_string());
        assert_eq!(contract.add_first(), "You already have more than one digital".to_string());

        assert_eq!(contract.next_digital(), 1);
    }

    #[test]
    fn test_pk() {
        let mut context = get_context("digital1.test".to_string(), 3_600_000_000_002);
        testing_env!(context.clone());
        let mut contract = DigitalCenter::new();

        assert_eq!(contract.add_first(), "success");

        context.signer_account_id = "digital2.test".to_string();
        testing_env!(context.clone());
        assert_eq!(contract.add_first(), "success");
        
        context.signer_account_id = "digital1.test".to_string();
        testing_env!(context.clone());
        assert_eq!(contract.add_first(), "You already have more than one digital");

        assert_eq!(contract.pk(0, 1), "you are lose, target get your digital!");

        assert_eq!(contract.pk(0, 1), "you are not own_digital owner");

        assert_eq!(contract.add_first(), "success");

        context.random_seed = vec![0, 1, 2, 3, 4, 5, 6, 7];
        testing_env!(context.clone());

        assert_eq!(contract.pk(2, 1), "you are win, target digital level minus 1!");
        assert_eq!(contract.pk(2, 1), "you are win, get target digital!");
        context.signer_account_id = "digital2.test".to_string();
        testing_env!(context.clone());
        assert_eq!(contract.pk(1, 2), "you are not own_digital owner");
    }

    #[test]
    fn test_pk_not_own() {
        let mut context = get_context("digital1.test".to_string(), 3_600_000_000_000);
        testing_env!(context.clone());
        let mut contract = DigitalCenter::new();

        assert_eq!(contract.add_first(), "success".to_string());

        context.signer_account_id = "digital2.test".to_string();
        testing_env!(context.clone());
        assert_eq!(contract.add_first(), "success".to_string());
        
        context.signer_account_id = "digital1.test".to_string();
        testing_env!(context.clone());
        assert_eq!(contract.pk(1, 0), "you are not own_digital owner");
    }

    #[test]
    fn get_digitals_work(){
        let mut context = get_context("digital1.test".to_string(), 3_600_000_000_000);
        testing_env!(context.clone());
        let mut contract = DigitalCenter::new();

        let aa: Vec<u64> = Vec::new();
        assert_eq!(contract.get_digitals("digital1.test".to_string()), aa);
        assert_eq!(contract.add_first(), "success".to_string());
        assert_eq!(contract.get_digitals("digital1.test".to_string()), vec![0]);
    }

    #[test]
    fn levelup(){
        let mut context = get_context("digital1.test".to_string(), 3_600_000_000_000);
        testing_env!(context.clone());
        let mut contract = DigitalCenter::new();
        assert_eq!(contract.add_first(), "success".to_string());
        context.attached_deposit = 50;
        testing_env!(context.clone());
        assert_eq!(contract.levelup(0), "levelup success");

        
    }

    #[test]
    #[should_panic(expected = "Deposit is too low")]
    fn levelup_panic(){
        let mut context = get_context("digital1.test".to_string(), 3_600_000_000_000);
        testing_env!(context.clone());
        let mut contract = DigitalCenter::new();
        context.attached_deposit = 5;
        testing_env!(context.clone());
        contract.levelup(0);
    }
}