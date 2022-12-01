use anchor_lang::prelude::*;
use std::convert::TryInto;
pub use switchboard_v2::{
    AggregatorAccountData, AggregatorHistoryBuffer, SwitchboardDecimal, SWITCHBOARD_PROGRAM_ID,
};

declare_id!("68V9BQJNemtzge1seCtrKWFHi4zc3NRyQv75MWTADvm4");

#[derive(Accounts)]
pub struct ReadHistory<'info> {
    #[account(
        has_one = history_buffer @ ErrorCode::InvalidHistoryBuffer
    )]
    pub aggregator: AccountLoader<'info, AggregatorAccountData>,
    /// CHECK: verified in the aggregator has_one check
    pub history_buffer: AccountInfo<'info>,
}

#[program]
pub mod volatility {

    use super::*;

    pub fn read_history(ctx: Context<ReadHistory>) -> anchor_lang::Result<()> {
        let history_buffer = AggregatorHistoryBuffer::new(&ctx.accounts.history_buffer)?;

        #[derive(Debug)]
        struct ClosingPrice {
            timestamp: i64,
            value: f64,
        }
        // get the closing prices for the 24hr intervals in the data in the history buffer
        let mut closing_prices: Vec<ClosingPrice> = Vec::new();
        // i've noticed the history buffer has data for the past five days only so we're going to get
        // closing prices for the past five days only, starting with today
        let now = Clock::get()?.unix_timestamp;
        let value_at_timestamp: f64 = history_buffer.lower_bound(now).unwrap().value.try_into()?;
        closing_prices.push(ClosingPrice {
            timestamp: now,
            value: value_at_timestamp,
        });
        let seconds_per_day: i64 = 3600 * 24;
        let mut next_timestamp = now - seconds_per_day;
        for _i in 1..=4 {
            let value: f64 = history_buffer
                .lower_bound(next_timestamp)
                .unwrap()
                .value
                .try_into()?;
            closing_prices.push(ClosingPrice {
                timestamp: next_timestamp,
                value,
            });
            next_timestamp -= seconds_per_day;
        }

        /* this function sorts the closing prices array by timestamp from lowest to largest.
        The closing prices above are from most recent to oldest but it makes more sence
        to calculate daily returns later with prices ordered from oldest to latest */
        fn sort_by_timestamp(data: &mut [ClosingPrice]) {
            data.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        }
        sort_by_timestamp(&mut closing_prices);

        // then we calculate the daily returns
        let mut daily_returns: Vec<f64> = Vec::new();
        for (i, closing_price) in closing_prices.iter().enumerate() {
            if i == 0 {
                continue;
            }
            let difference = closing_price.value - closing_prices[i - 1].value;
            let daily_return = difference / closing_prices[i - 1].value;
            let percentage_daily_return = daily_return * 100.0;
            daily_returns.push(percentage_daily_return)
        }
        // calculate average return
        let mut daily_returns_sum = 0.0;
        let mut daily_returns_len = 0.0;
        for i in daily_returns.iter() {
            daily_returns_sum += i;
            daily_returns_len += 1.0;
        }
        let average_return = daily_returns_sum / daily_returns_len;
        // calculate variance
        let mut total: f64 = 0.0;
        for i in daily_returns.iter() {
            let return_minus_mean = i - average_return;
            total += return_minus_mean.powf(2.0)
        }
        let variance = total / (daily_returns_len - 1.0);
        // calculate volatility/standard deviation
        let standard_deviation = variance.sqrt();

        msg!("closing prices {:?}!", closing_prices);
        msg!("daily returns {:?}!", daily_returns);
        msg!("average return {:?}!", average_return);
        msg!("variance {:?}!", variance);
        msg!("Volatility {:?}!", standard_deviation);

        Ok(())
    }
}

#[error_code]
#[derive(Eq, PartialEq)]
pub enum ErrorCode {
    #[msg("Not a valid Switchboard account")]
    InvalidSwitchboardAccount,
    #[msg("History buffer mismatch")]
    InvalidHistoryBuffer,
    #[msg("History buffer is empty")]
    EmptyHistoryBuffer,
}
