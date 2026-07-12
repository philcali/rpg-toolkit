use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::condition::BranchCondition;
use crate::error::CommonError;
use crate::item::ItemId;

pub type ShopId = String;

/// Resource inserted by the renderer when transitioning to AppPhase::Shop.
/// Contains the shop ID that the Shop Scene plugin will use to load the shop definition.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ActiveShopId {
    pub shop_id: ShopId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShopEntry {
    pub item_id: ItemId,
    pub buy_price: u32,
    pub sell_price: Option<u32>,
    pub stock_limit: Option<u32>,
    pub condition: Option<BranchCondition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShopDefinition {
    pub id: ShopId,
    pub display_name: String,
    pub entries: Vec<ShopEntry>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ShopRegistry {
    pub shops: HashMap<ShopId, ShopDefinition>,
}

/// Validates a shop display name: must be 1–64 characters after trimming.
fn validate_shop_name(name: &str) -> Result<String, CommonError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(CommonError::ShopValidationError(
            "Display name must contain at least 1 non-whitespace character".to_string(),
        ));
    }
    if trimmed.len() > 64 {
        return Err(CommonError::ShopValidationError(
            "Display name must not exceed 64 characters".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

impl ShopRegistry {
    /// Creates a new shop with the given name, assigns a UUID v4 ID, and inserts it.
    /// Returns the generated ShopId on success.
    pub fn create_shop(&mut self, name: &str) -> Result<ShopId, CommonError> {
        let display_name = validate_shop_name(name)?;
        let id = Uuid::new_v4().to_string();

        let definition = ShopDefinition {
            id: id.clone(),
            display_name,
            entries: Vec::new(),
        };

        self.shops.insert(id.clone(), definition);
        Ok(id)
    }

    /// Removes a shop by ID. Returns an error if the shop is not found.
    pub fn delete_shop(&mut self, id: &ShopId) -> Result<(), CommonError> {
        if self.shops.remove(id).is_none() {
            return Err(CommonError::ShopValidationError(format!(
                "Shop with id '{}' not found",
                id
            )));
        }
        Ok(())
    }

    /// Renames a shop after validating the new name (1–64 trimmed chars).
    /// Returns an error if the shop is not found or the name is invalid.
    pub fn rename_shop(&mut self, id: &ShopId, name: &str) -> Result<(), CommonError> {
        let display_name = validate_shop_name(name)?;
        let shop = self.shops.get_mut(id).ok_or_else(|| {
            CommonError::ShopValidationError(format!("Shop with id '{}' not found", id))
        })?;
        shop.display_name = display_name;
        Ok(())
    }

    /// Adds an entry to a shop. Rejects duplicates (same ItemId) and enforces max 256 entries.
    pub fn add_entry(&mut self, shop_id: &ShopId, entry: ShopEntry) -> Result<(), CommonError> {
        let shop = self.shops.get_mut(shop_id).ok_or_else(|| {
            CommonError::ShopValidationError(format!("Shop with id '{}' not found", shop_id))
        })?;

        // Check for duplicate ItemId
        if shop.entries.iter().any(|e| e.item_id == entry.item_id) {
            return Err(CommonError::ShopValidationError(format!(
                "Item '{}' already exists in this shop",
                entry.item_id
            )));
        }

        // Enforce max 256 entries
        if shop.entries.len() >= 256 {
            return Err(CommonError::ShopValidationError(
                "Shop cannot have more than 256 entries".to_string(),
            ));
        }

        shop.entries.push(entry);
        Ok(())
    }

    /// Removes an entry from a shop by item ID.
    /// Returns an error if the shop or entry is not found.
    pub fn remove_entry(&mut self, shop_id: &ShopId, item_id: &ItemId) -> Result<(), CommonError> {
        let shop = self.shops.get_mut(shop_id).ok_or_else(|| {
            CommonError::ShopValidationError(format!("Shop with id '{}' not found", shop_id))
        })?;

        let pos = shop
            .entries
            .iter()
            .position(|e| &e.item_id == item_id)
            .ok_or_else(|| {
                CommonError::ShopValidationError(format!(
                    "Entry with item id '{}' not found in shop",
                    item_id
                ))
            })?;

        shop.entries.remove(pos);
        Ok(())
    }

    /// Updates an existing entry in a shop identified by item_id.
    /// Allows updating buy_price, sell_price, stock_limit, and condition.
    pub fn update_entry(
        &mut self,
        shop_id: &ShopId,
        item_id: &ItemId,
        buy_price: Option<u32>,
        sell_price: Option<Option<u32>>,
        stock_limit: Option<Option<u32>>,
        condition: Option<Option<BranchCondition>>,
    ) -> Result<(), CommonError> {
        let shop = self.shops.get_mut(shop_id).ok_or_else(|| {
            CommonError::ShopValidationError(format!("Shop with id '{}' not found", shop_id))
        })?;

        let entry = shop
            .entries
            .iter_mut()
            .find(|e| &e.item_id == item_id)
            .ok_or_else(|| {
                CommonError::ShopValidationError(format!(
                    "Entry with item id '{}' not found in shop",
                    item_id
                ))
            })?;

        if let Some(price) = buy_price {
            entry.buy_price = price;
        }
        if let Some(sell) = sell_price {
            entry.sell_price = sell;
        }
        if let Some(stock) = stock_limit {
            entry.stock_limit = stock;
        }
        if let Some(cond) = condition {
            entry.condition = cond;
        }

        Ok(())
    }

    /// Returns all shops sorted case-insensitively by display name.
    pub fn sorted_shops(&self) -> Vec<&ShopDefinition> {
        let mut shops: Vec<&ShopDefinition> = self.shops.values().collect();
        shops.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        });
        shops
    }

    /// Returns shops whose display name contains the query substring (case-insensitive).
    pub fn search_shops(&self, query: &str) -> Vec<&ShopDefinition> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<&ShopDefinition> = self
            .shops
            .values()
            .filter(|shop| shop.display_name.to_lowercase().contains(&query_lower))
            .collect();
        results.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        });
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_shop_valid_name() {
        let mut registry = ShopRegistry::default();
        let result = registry.create_shop("Blacksmith");
        assert!(result.is_ok());
        let id = result.unwrap();
        assert_eq!(registry.shops.len(), 1);
        assert_eq!(registry.shops[&id].display_name, "Blacksmith");
    }

    #[test]
    fn test_create_shop_trims_name() {
        let mut registry = ShopRegistry::default();
        let id = registry.create_shop("  Potion Shop  ").unwrap();
        assert_eq!(registry.shops[&id].display_name, "Potion Shop");
    }

    #[test]
    fn test_create_shop_empty_name_rejected() {
        let mut registry = ShopRegistry::default();
        let result = registry.create_shop("   ");
        assert!(result.is_err());
        assert_eq!(registry.shops.len(), 0);
    }

    #[test]
    fn test_create_shop_name_too_long_rejected() {
        let mut registry = ShopRegistry::default();
        let long_name = "a".repeat(65);
        let result = registry.create_shop(&long_name);
        assert!(result.is_err());
        assert_eq!(registry.shops.len(), 0);
    }

    #[test]
    fn test_create_shop_name_exactly_64_chars() {
        let mut registry = ShopRegistry::default();
        let name = "a".repeat(64);
        let result = registry.create_shop(&name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_shop_success() {
        let mut registry = ShopRegistry::default();
        let id = registry.create_shop("Shop").unwrap();
        assert!(registry.delete_shop(&id).is_ok());
        assert_eq!(registry.shops.len(), 0);
    }

    #[test]
    fn test_delete_shop_not_found() {
        let mut registry = ShopRegistry::default();
        let result = registry.delete_shop(&"nonexistent".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_shop_success() {
        let mut registry = ShopRegistry::default();
        let id = registry.create_shop("Old Name").unwrap();
        assert!(registry.rename_shop(&id, "New Name").is_ok());
        assert_eq!(registry.shops[&id].display_name, "New Name");
    }

    #[test]
    fn test_rename_shop_not_found() {
        let mut registry = ShopRegistry::default();
        let result = registry.rename_shop(&"nonexistent".to_string(), "Name");
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_shop_invalid_name() {
        let mut registry = ShopRegistry::default();
        let id = registry.create_shop("Shop").unwrap();
        let result = registry.rename_shop(&id, "");
        assert!(result.is_err());
        // Name should remain unchanged
        assert_eq!(registry.shops[&id].display_name, "Shop");
    }

    #[test]
    fn test_add_entry_success() {
        let mut registry = ShopRegistry::default();
        let id = registry.create_shop("Shop").unwrap();
        let entry = ShopEntry {
            item_id: "item-1".to_string(),
            buy_price: 100,
            sell_price: None,
            stock_limit: None,
            condition: None,
        };
        assert!(registry.add_entry(&id, entry).is_ok());
        assert_eq!(registry.shops[&id].entries.len(), 1);
    }

    #[test]
    fn test_add_entry_duplicate_rejected() {
        let mut registry = ShopRegistry::default();
        let id = registry.create_shop("Shop").unwrap();
        let entry1 = ShopEntry {
            item_id: "item-1".to_string(),
            buy_price: 100,
            sell_price: None,
            stock_limit: None,
            condition: None,
        };
        let entry2 = ShopEntry {
            item_id: "item-1".to_string(),
            buy_price: 200,
            sell_price: Some(50),
            stock_limit: Some(10),
            condition: None,
        };
        registry.add_entry(&id, entry1).unwrap();
        let result = registry.add_entry(&id, entry2);
        assert!(result.is_err());
        // Original entry unchanged
        assert_eq!(registry.shops[&id].entries.len(), 1);
        assert_eq!(registry.shops[&id].entries[0].buy_price, 100);
    }

    #[test]
    fn test_add_entry_max_256_enforced() {
        let mut registry = ShopRegistry::default();
        let id = registry.create_shop("Shop").unwrap();
        for i in 0..256 {
            let entry = ShopEntry {
                item_id: format!("item-{}", i),
                buy_price: 10,
                sell_price: None,
                stock_limit: None,
                condition: None,
            };
            registry.add_entry(&id, entry).unwrap();
        }
        let entry = ShopEntry {
            item_id: "item-256".to_string(),
            buy_price: 10,
            sell_price: None,
            stock_limit: None,
            condition: None,
        };
        let result = registry.add_entry(&id, entry);
        assert!(result.is_err());
        assert_eq!(registry.shops[&id].entries.len(), 256);
    }

    #[test]
    fn test_remove_entry_success() {
        let mut registry = ShopRegistry::default();
        let id = registry.create_shop("Shop").unwrap();
        let entry = ShopEntry {
            item_id: "item-1".to_string(),
            buy_price: 100,
            sell_price: None,
            stock_limit: None,
            condition: None,
        };
        registry.add_entry(&id, entry).unwrap();
        assert!(registry.remove_entry(&id, &"item-1".to_string()).is_ok());
        assert_eq!(registry.shops[&id].entries.len(), 0);
    }

    #[test]
    fn test_remove_entry_not_found() {
        let mut registry = ShopRegistry::default();
        let id = registry.create_shop("Shop").unwrap();
        let result = registry.remove_entry(&id, &"nonexistent".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_update_entry_success() {
        let mut registry = ShopRegistry::default();
        let id = registry.create_shop("Shop").unwrap();
        let entry = ShopEntry {
            item_id: "item-1".to_string(),
            buy_price: 100,
            sell_price: None,
            stock_limit: None,
            condition: None,
        };
        registry.add_entry(&id, entry).unwrap();
        assert!(
            registry
                .update_entry(
                    &id,
                    &"item-1".to_string(),
                    Some(200),
                    Some(Some(50)),
                    Some(Some(10)),
                    None,
                )
                .is_ok()
        );
        let updated = &registry.shops[&id].entries[0];
        assert_eq!(updated.buy_price, 200);
        assert_eq!(updated.sell_price, Some(50));
        assert_eq!(updated.stock_limit, Some(10));
    }

    #[test]
    fn test_update_entry_not_found() {
        let mut registry = ShopRegistry::default();
        let id = registry.create_shop("Shop").unwrap();
        let result =
            registry.update_entry(&id, &"nonexistent".to_string(), Some(100), None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_sorted_shops_case_insensitive() {
        let mut registry = ShopRegistry::default();
        registry.create_shop("Zephyr").unwrap();
        registry.create_shop("alpha").unwrap();
        registry.create_shop("Beta").unwrap();
        let sorted = registry.sorted_shops();
        assert_eq!(sorted[0].display_name, "alpha");
        assert_eq!(sorted[1].display_name, "Beta");
        assert_eq!(sorted[2].display_name, "Zephyr");
    }

    #[test]
    fn test_search_shops_substring() {
        let mut registry = ShopRegistry::default();
        registry.create_shop("Blacksmith").unwrap();
        registry.create_shop("Potion Shop").unwrap();
        registry.create_shop("Magic Emporium").unwrap();

        let results = registry.search_shops("shop");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].display_name, "Potion Shop");
    }

    #[test]
    fn test_search_shops_case_insensitive() {
        let mut registry = ShopRegistry::default();
        registry.create_shop("ARMOR Shop").unwrap();
        registry.create_shop("armor emporium").unwrap();

        let results = registry.search_shops("ARMOR");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_shops_empty_query_returns_all() {
        let mut registry = ShopRegistry::default();
        registry.create_shop("Shop A").unwrap();
        registry.create_shop("Shop B").unwrap();
        let results = registry.search_shops("");
        assert_eq!(results.len(), 2);
    }
}
