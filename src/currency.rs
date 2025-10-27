use std::collections::HashMap;
use reqwest;
use serde::Deserialize;

#[derive(Deserialize)]
struct ExchangeRateResponse {
    rates: HashMap<String, f64>,
}

#[derive(Debug)]
pub struct CurrencyConverter {
    rates: HashMap<String, f64>,
}

impl CurrencyConverter {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut rates = HashMap::new();

        // Try to fetch from API
        match Self::fetch_rates().await {
            Ok(api_rates) => {
                rates = api_rates;
            }
            Err(e) => {
                eprintln!("Failed to fetch currency rates: {}. Using fallback rates.", e);
                // Fallback rates
                rates.insert("USD".to_string(), 1.0);
                rates.insert("EUR".to_string(), 0.92);
                rates.insert("INR".to_string(), 83.50);
            }
        }

        Ok(Self { rates })
    }

    async fn fetch_rates() -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
        let url = "https://api.exchangerate-api.com/v4/latest/USD";
        let response = reqwest::get(url).await?;
        let data: ExchangeRateResponse = response.json().await?;
        Ok(data.rates)
    }

    pub fn convert(&self, amount: f64, from: &str, to: &str) -> Result<f64, String> {
        let from_rate = self
            .rates
            .get(from)
            .ok_or_else(|| format!("Unknown currency: {}", from))?;
        let to_rate = self
            .rates
            .get(to)
            .ok_or_else(|| format!("Unknown currency: {}", to))?;

        // Convert to USD first, then to target currency
        let usd_amount = amount / from_rate;
        let result = usd_amount * to_rate;

        Ok(result)
    }
}
