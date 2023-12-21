use std::time::{Duration, Instant};

pub struct VariablePredictor {
	pub total: usize,
	history: Vec<(u8, Instant)>,
	middle_check_duration: Duration,
}

impl VariablePredictor {
	pub fn new(total: usize, middle_check_duration: Duration) -> Self {
		VariablePredictor {
			total,
			middle_check_duration,
			history: Vec::new(),
		}
	}

	pub fn record_request(&mut self, request: u8) {
		let timestamp = Instant::now();
		self.history.push((request, timestamp));
	}

	pub fn predict_time_to_zero(&self) -> Option<Duration> {
		if self.history.is_empty() {
			return None;
		}

		let total_requests = self.history.len();
		let remaining_requests = self.total as f64 - total_requests as f64;

		let now = Instant::now();
		let start_time = self.history[0].1;
		let average_request_rate =
			if now.duration_since(start_time) < self.middle_check_duration / 5 {
				0.0
			} else {
				total_requests as f64 / now.duration_since(start_time).as_millis() as f64
			};

		// calculate last 10 seconds average request rate
		let mut middle_check_requests = 0;
		for (_, timestamp) in self.history.iter().rev() {
			if now.duration_since(*timestamp) > self.middle_check_duration {
				break;
			}
			middle_check_requests += 1;
		}
		let avarage_request_rate_last_of_middle_check =
			middle_check_requests as f64 / self.middle_check_duration.as_millis() as f64;

		debug!(
			"total requests: {}, remaining requests: {}, average request rate: {}, last 10 seconds average request rate: {}",
			total_requests, remaining_requests, average_request_rate, avarage_request_rate_last_of_middle_check
		);

		let general_predict = if average_request_rate == 0.0 {
			Duration::from_millis(u64::MAX)
		} else {
			let time_to_zero = remaining_requests / average_request_rate;
			Duration::from_millis(time_to_zero as u64)
		};
		let last_10_seconds_predict = if avarage_request_rate_last_of_middle_check == 0.0 {
			Duration::from_millis(u64::MAX)
		} else {
			let time_to_zero = remaining_requests / avarage_request_rate_last_of_middle_check;
			Duration::from_millis(time_to_zero as u64)
		};

		debug!(
			"predict time to zero: general: {:?}, last 10 seconds: {:?}",
			general_predict, last_10_seconds_predict
		);
		Some(general_predict.min(last_10_seconds_predict))
	}
}

#[cfg(all(test, feature = "__test"))]
mod tests {
	use super::*;
	use std::thread::sleep;

	#[test]
	fn predict_time_to_zero_works() {
		let mut predictor = VariablePredictor::new(20, Duration::from_millis(10));
		assert!(predictor.predict_time_to_zero().is_none());

		for i in 0..10 {
			predictor.record_request(i);
			sleep(Duration::from_millis(1));
		}
		let predict_duration = predictor.predict_time_to_zero().unwrap();
		assert!(predict_duration >= Duration::from_millis(10));
		// the max value set to 14 because we assume unit test run in a few mill-seconds
		assert!(predict_duration <= Duration::from_millis(14));

		let mut predictor = VariablePredictor::new(20, Duration::from_millis(10));
		for i in 0..15 {
			predictor.record_request(i);
			sleep(Duration::from_millis(1));
		}
		let predict_duration = predictor.predict_time_to_zero().unwrap();
		assert!(predict_duration >= Duration::from_millis(5));
		// the max value set to 14 because we assume unit test run in a few mill-seconds
		assert!(predict_duration <= Duration::from_millis(7));

		let mut predictor = VariablePredictor::new(20, Duration::from_millis(10));
		for i in 0..5 {
			predictor.record_request(i);
			sleep(Duration::from_millis(1));
		}
		let predict_duration = predictor.predict_time_to_zero().unwrap();
		assert!(predict_duration >= Duration::from_millis(15));
		// the max value set to 14 because we assume unit test run in a few mill-seconds
		assert!(predict_duration <= Duration::from_millis(19));

		let mut predictor = VariablePredictor::new(20, Duration::from_millis(10));
		for i in 0..5 {
			predictor.record_request(i);
			sleep(Duration::from_millis(5));
		}
		// the next 10 requests will be faster than before
		for i in 5..15 {
			predictor.record_request(i);
			sleep(Duration::from_millis(1));
		}
		let predict_duration = predictor.predict_time_to_zero().unwrap();
		assert!(predict_duration >= Duration::from_millis(5));
		// the max value set to 14 because we assume unit test run in a few mill-seconds
		assert!(predict_duration <= Duration::from_millis(7));

		let mut predictor = VariablePredictor::new(20, Duration::from_millis(10));
		for i in 0..5 {
			predictor.record_request(i);
			sleep(Duration::from_millis(1));
		}
		// the next 10 requests will be faster than before
		for i in 5..15 {
			predictor.record_request(i);
			sleep(Duration::from_millis(4));
		}
		let predict_duration = predictor.predict_time_to_zero().unwrap();
		assert!(predict_duration >= Duration::from_millis(5 * 3));
		// the max value set to 19 because we assume unit test run in a few mill-seconds
		assert!(predict_duration <= Duration::from_millis(5 * 3 + 4));
	}
}
