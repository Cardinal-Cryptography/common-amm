use amm_helpers::{
    constants::stable_pool::{MAX_AMP, MAX_AMP_CHANGE, MIN_AMP, MIN_RAMP_DURATION},
    ensure,
};
use ink::env::DefaultEnvironment;
use traits::{MathError, StablePoolError};

#[derive(Default, Debug, scale::Encode, scale::Decode, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct AmpCoef {
    /// Initial amplification coefficient.
    init_amp_coef: u128,
    /// Target for ramping up amplification coefficient.
    future_amp_coef: u128,
    /// Initial amplification time.
    init_time: u64,
    /// Stop ramp up amplification time.
    future_time: u64,
}

impl AmpCoef {
    pub fn new(init_amp_coef: u128) -> Result<Self, StablePoolError> {
        ensure!(init_amp_coef >= MIN_AMP, StablePoolError::AmpCoefTooLow);
        ensure!(init_amp_coef <= MAX_AMP, StablePoolError::AmpCoefTooHigh);
        Ok(Self {
            init_amp_coef,
            future_amp_coef: init_amp_coef,
            init_time: 0,
            future_time: 0,
        })
    }

    pub fn compute_amp_coef(&self) -> Result<u128, MathError> {
        let current_time = ink::env::block_timestamp::<DefaultEnvironment>();
        if current_time < self.future_time {
            let time_range = self
                .future_time
                .checked_sub(self.init_time)
                .ok_or(MathError::SubUnderflow(51))?;
            let time_delta = current_time
                .checked_sub(self.init_time)
                .ok_or(MathError::SubUnderflow(52))?;

            // Compute amp factor based on ramp time
            let amp_range = self.future_amp_coef.abs_diff(self.init_amp_coef);
            let amp_delta = amp_range
                .checked_mul(time_delta as u128)
                .ok_or(MathError::MulOverflow(51))?
                .checked_div(time_range as u128)
                .ok_or(MathError::DivByZero(51))?;
            if self.future_amp_coef >= self.init_amp_coef {
                // Ramp up
                self.init_amp_coef
                    .checked_add(amp_delta)
                    .ok_or(MathError::AddOverflow(1))
            } else {
                // Ramp down
                self.init_amp_coef
                    .checked_sub(amp_delta)
                    .ok_or(MathError::SubUnderflow(55))
            }
        } else {
            Ok(self.future_amp_coef)
        }
    }

    pub fn ramp_amp_coef(
        &mut self,
        future_amp_coef: u128,
        future_time: u64,
    ) -> Result<(), StablePoolError> {
        ensure!(future_amp_coef >= MIN_AMP, StablePoolError::AmpCoefTooLow);
        ensure!(future_amp_coef <= MAX_AMP, StablePoolError::AmpCoefTooHigh);
        let current_time = ink::env::block_timestamp::<DefaultEnvironment>();
        let ramp_duration = future_time.checked_sub(current_time);
        ensure!(
            ramp_duration.is_some() && ramp_duration.unwrap() >= MIN_RAMP_DURATION,
            StablePoolError::AmpCoefRampDurationTooShort
        );
        let current_amp_coef = self.compute_amp_coef()?;
        ensure!(
            (future_amp_coef >= current_amp_coef
                && future_amp_coef <= current_amp_coef * MAX_AMP_CHANGE)
                || (future_amp_coef < current_amp_coef
                    && future_amp_coef * MAX_AMP_CHANGE >= current_amp_coef),
            StablePoolError::AmpCoefChangeTooLarge
        );
        self.init_amp_coef = current_amp_coef;
        self.init_time = current_time;
        self.future_amp_coef = future_amp_coef;
        self.future_time = future_time;
        Ok(())
    }

    /// Stop ramping A. If ramping is not in progress, it does not influence the A.
    pub fn stop_ramp_amp_coef(&mut self) -> Result<(), StablePoolError> {
        let current_amp_coef = self.compute_amp_coef()?;
        let current_time = ink::env::block_timestamp::<DefaultEnvironment>();
        self.init_amp_coef = current_amp_coef;
        self.future_amp_coef = current_amp_coef;
        self.init_time = current_time;
        self.future_time = current_time;
        Ok(())
    }

    /// Returns a tuple of the future amplification coefficient and the ramping end time.
    /// Returns `None` if the amplification coefficient is not in ramping period.
    pub fn future_amp_coef(&self) -> Option<(u128, u64)> {
        let current_time = ink::env::block_timestamp::<DefaultEnvironment>();
        if current_time < self.future_time {
            Some((self.future_amp_coef, self.future_time))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_block_timestamp(ts: u64) {
        ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(ts);
    }

    #[test]
    fn amp_coef_up() {
        let amp_coef = AmpCoef {
            init_amp_coef: 100,
            future_amp_coef: 1000,
            init_time: 100,
            future_time: 1600,
        };
        set_block_timestamp(100);
        assert_eq!(amp_coef.compute_amp_coef(), Ok(100));
        set_block_timestamp(850);
        assert_eq!(amp_coef.compute_amp_coef(), Ok(550));
        set_block_timestamp(1600);
        assert_eq!(amp_coef.compute_amp_coef(), Ok(1000));
    }

    #[test]
    fn amp_coef_down() {
        let amp_coef = AmpCoef {
            init_amp_coef: 1000,
            future_amp_coef: 100,
            init_time: 100,
            future_time: 1600,
        };
        set_block_timestamp(100);
        assert_eq!(amp_coef.compute_amp_coef(), Ok(1000));
        set_block_timestamp(850);
        assert_eq!(amp_coef.compute_amp_coef(), Ok(550));
        set_block_timestamp(1600);
        assert_eq!(amp_coef.compute_amp_coef(), Ok(100));
    }

    #[test]
    fn amp_coef_change_duration() {
        set_block_timestamp(1000);
        let mut amp_coef = AmpCoef {
            init_amp_coef: 1000,
            future_amp_coef: 100,
            init_time: 100,
            future_time: 1600,
        };
        assert_eq!(
            amp_coef.ramp_amp_coef(1000, 999),
            Err(StablePoolError::AmpCoefRampDurationTooShort)
        );
        assert_eq!(
            amp_coef.ramp_amp_coef(1000, 1000 + MIN_RAMP_DURATION - 1),
            Err(StablePoolError::AmpCoefRampDurationTooShort)
        );
        assert_eq!(
            amp_coef.ramp_amp_coef(1000, 1000 + MIN_RAMP_DURATION),
            Ok(())
        );
    }

    #[test]
    fn amp_coef_change_too_large() {
        set_block_timestamp(100);
        let mut amp_coef = AmpCoef {
            init_amp_coef: 100,
            future_amp_coef: 100,
            init_time: 100,
            future_time: 1600,
        };
        assert_eq!(
            amp_coef.ramp_amp_coef(1001, 100 + MIN_RAMP_DURATION),
            Err(StablePoolError::AmpCoefChangeTooLarge)
        );
        assert_eq!(
            amp_coef.ramp_amp_coef(1000, 100 + MIN_RAMP_DURATION),
            Ok(())
        );
    }

    #[test]
    fn amp_coef_stop_ramp() {
        set_block_timestamp(100);
        let mut amp_coef = AmpCoef {
            init_amp_coef: 100,
            future_amp_coef: 100,
            init_time: 100,
            future_time: 1600,
        };
        assert_eq!(amp_coef.compute_amp_coef(), Ok(100));
        assert_eq!(
            amp_coef.ramp_amp_coef(1000, 100 + MIN_RAMP_DURATION),
            Ok(())
        );
        set_block_timestamp(100 + MIN_RAMP_DURATION / 2);
        assert!(amp_coef.stop_ramp_amp_coef().is_ok());
        assert_eq!(amp_coef.compute_amp_coef(), Ok(550));
    }

    #[test]
    fn amp_coef_stop_ramp_no_change() {
        set_block_timestamp(100);
        let mut amp_coef = AmpCoef {
            init_amp_coef: 100,
            future_amp_coef: 100,
            init_time: 100,
            future_time: 1600,
        };
        assert_eq!(amp_coef.compute_amp_coef(), Ok(100));
        assert_eq!(
            amp_coef.ramp_amp_coef(1000, 100 + MIN_RAMP_DURATION),
            Ok(())
        );
        set_block_timestamp(100 + MIN_RAMP_DURATION);
        assert_eq!(amp_coef.compute_amp_coef(), Ok(1000));
        set_block_timestamp(100 + MIN_RAMP_DURATION * 2);
        assert!(amp_coef.stop_ramp_amp_coef().is_ok());
        assert_eq!(amp_coef.compute_amp_coef(), Ok(1000));
    }
}
