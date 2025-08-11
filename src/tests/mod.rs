#[cfg(test)]
mod tests {
    use super::*;
    use alkanes_runtime::test_utils::*;
    use alkanes_support::test_utils::*;
    use anyhow::Result;

    // Helper to create a test token contract
    fn setup_test_token() -> Result<BondingCurveToken> {
        let token = BondingCurveToken::default();
        
        // Initialize with test parameters
        token.initialize(
            "Test".as_bytes().to_vec().into(),  // name_part1
            "Token".as_bytes().to_vec().into(), // name_part2
            "TST".as_bytes().to_vec().into(),   // symbol
            1_000_000,                          // base_price (0.01 BUSD)
            1500,                               // growth_rate (1.5%)
            10_000_000_000_000,                // graduation_threshold (100k BUSD)
            0,                                  // base_token_type (BUSD)
            1_000_000_000_000_000,             // max_supply (1B)
            0,                                  // lp_distribution_strategy (FullBurn)
        )?;
        
        Ok(token)
    }

    #[test]
    fn test_initialization() -> Result<()> {
        let token = setup_test_token()?;
        
        // Check name
        let name = String::from_utf8(token.name_pointer().get().as_ref().to_vec())?;
        assert_eq!(name, "TestToken");
        
        // Check symbol
        let symbol = String::from_utf8(token.symbol_pointer().get().as_ref().to_vec())?;
        assert_eq!(symbol, "TST");
        
        // Check initial state
        assert_eq!(token.total_supply_pointer().get_value::<u128>(), 0);
        assert_eq!(token.base_reserves_pointer().get_value::<u128>(), 0);
        assert_eq!(token.graduated_pointer().get_value::<u8>(), 0);
        
        // Check curve parameters
        let params_data = token.curve_params_pointer().get();
        let params: CurveParams = serde_json::from_slice(params_data.as_ref())?;
        assert_eq!(params.base_price, 1_000_000);
        assert_eq!(params.growth_rate, 1500);
        assert_eq!(params.base_token, BaseToken::BUSD);
        
        Ok(())
    }

    #[test]
    fn test_buy_tokens() -> Result<()> {
        let token = setup_test_token()?;
        
        // Buy 1000 tokens
        let min_tokens = 1000;
        token.buy_tokens(min_tokens)?;
        
        // Check supply increased
        assert_eq!(token.total_supply_pointer().get_value::<u128>(), min_tokens);
        
        // Check reserves increased
        let params_data = token.curve_params_pointer().get();
        let params: CurveParams = serde_json::from_slice(params_data.as_ref())?;
        let expected_reserves = min_tokens * params.base_price;
        assert_eq!(token.base_reserves_pointer().get_value::<u128>(), expected_reserves);
        
        Ok(())
    }

    #[test]
    fn test_sell_tokens() -> Result<()> {
        let token = setup_test_token()?;
        
        // First buy some tokens
        let tokens = 1000;
        token.buy_tokens(tokens)?;
        
        // Now sell half
        let sell_amount = 500;
        let min_base_out = sell_amount * 1_000_000; // base_price
        token.sell_tokens(sell_amount, min_base_out)?;
        
        // Check supply decreased
        assert_eq!(token.total_supply_pointer().get_value::<u128>(), 500);
        
        // Check reserves decreased
        assert_eq!(token.base_reserves_pointer().get_value::<u128>(), min_base_out);
        
        Ok(())
    }

    #[test]
    fn test_graduation() -> Result<()> {
        let token = setup_test_token()?;
        
        // Buy enough tokens to reach graduation threshold
        let params_data = token.curve_params_pointer().get();
        let params: CurveParams = serde_json::from_slice(params_data.as_ref())?;
        let tokens_needed = params.graduation_threshold / params.base_price;
        token.buy_tokens(tokens_needed)?;
        
        // Try to graduate
        token.graduate()?;
        
        // Check graduation state
        assert_eq!(token.graduated_pointer().get_value::<u8>(), 1);
        assert!(token.amm_pool_pointer().get_value::<u128>() > 0);
        
        Ok(())
    }

    #[test]
    #[should_panic(expected = "Market cap below graduation threshold")]
    fn test_graduation_fails_below_threshold() {
        let token = setup_test_token().unwrap();
        
        // Buy some tokens but not enough
        token.buy_tokens(1000).unwrap();
        
        // Try to graduate (should fail)
        token.graduate().unwrap();
    }

    #[test]
    fn test_price_quotes() -> Result<()> {
        let token = setup_test_token()?;
        
        // Get buy quote
        let amount = 1000;
        let response = token.get_buy_quote(amount)?;
        let quote = u128::from_le_bytes(response.data.try_into().unwrap());
        
        // Check quote matches expected price
        let params_data = token.curve_params_pointer().get();
        let params: CurveParams = serde_json::from_slice(params_data.as_ref())?;
        let expected_cost = amount * params.base_price;
        assert_eq!(quote, expected_cost);
        
        // Get sell quote
        let response = token.get_sell_quote(amount)?;
        let quote = u128::from_le_bytes(response.data.try_into().unwrap());
        assert_eq!(quote, expected_cost); // Should match for now (simplified)
        
        Ok(())
    }

    #[test]
    fn test_view_functions() -> Result<()> {
        let token = setup_test_token()?;
        
        // Test name
        let response = token.get_name()?;
        let name = String::from_utf8(response.data)?;
        assert_eq!(name, "TestToken");
        
        // Test symbol
        let response = token.get_symbol()?;
        let symbol = String::from_utf8(response.data)?;
        assert_eq!(symbol, "TST");
        
        // Test total supply
        let response = token.get_total_supply()?;
        let supply = u128::from_le_bytes(response.data.try_into().unwrap());
        assert_eq!(supply, 0);
        
        // Test base reserves
        let response = token.get_base_reserves()?;
        let reserves = u128::from_le_bytes(response.data.try_into().unwrap());
        assert_eq!(reserves, 0);
        
        // Test graduation state
        let response = token.is_graduated()?;
        assert_eq!(response.data[0], 0);
        
        Ok(())
    }

    #[test]
    fn test_curve_state() -> Result<()> {
        let token = setup_test_token()?;
        
        let response = token.get_curve_state()?;
        let state: serde_json::Value = serde_json::from_slice(&response.data)?;
        
        assert_eq!(state["base_price"], 1_000_000);
        assert_eq!(state["growth_rate"], 1500);
        assert_eq!(state["base_token"], "BUSD");
        assert_eq!(state["current_supply"], 0);
        assert_eq!(state["graduated"], false);
        
        Ok(())
    }

    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_double_initialization() {
        let token = setup_test_token().unwrap();
        
        // Try to initialize again (should fail)
        token.initialize(
            "Test2".as_bytes().to_vec().into(),
            "Token2".as_bytes().to_vec().into(),
            "TST2".as_bytes().to_vec().into(),
            2_000_000,
            2000,
            20_000_000_000_000,
            0,
            2_000_000_000_000_000,
            1,
        ).unwrap();
    }

    #[test]
    #[should_panic(expected = "Bonding curve has graduated")]
    fn test_trading_after_graduation() {
        let token = setup_test_token().unwrap();
        
        // Buy enough to graduate
        let params_data = token.curve_params_pointer().get().as_ref().to_vec();
        let params: CurveParams = serde_json::from_slice(&params_data).unwrap();
        let tokens_needed = params.graduation_threshold / params.base_price;
        token.buy_tokens(tokens_needed).unwrap();
        
        // Graduate
        token.graduate().unwrap();
        
        // Try to buy more (should fail)
        token.buy_tokens(1000).unwrap();
    }
}