use crate::*;
use frame_support::pallet_prelude::*;

impl<T: Config> Pallet<T> {
	/// Fetch current price and return the result in cents.
	pub fn fetch_property() -> Result<PropertyInfoData<T>, http::Error> {
		// We want to keep the offchain worker execution time reasonable, so we set a hard-coded
		// deadline to 2s to complete the external call.
		// You can also wait indefinitely for the response, however you may still get a timeout
		// coming from the host machine.
		let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
		// Initiate an external HTTP GET request.
		// This is using high-level wrappers from `sp_runtime`, for the low-level calls that
		// you can find in `sp_io`. The API is trying to be similar to `request`, but
		// since we are running in a custom WASM execution environment we can't simply
		// import the library here.

		let request = http::Request::get(
        "https://ipfs.io/ipfs/QmZ3Dn5B2UMuv9PFr1Ba3NGSKft2rwToBKCPaCTCmSab4k?filename=testing_data.json"
    );

		// We set the deadline for sending of the request, note that awaiting response can
		// have a separate deadline. Next we send the request, before that it's also possible
		// to alter request headers or stream body content in case of non-GET requests.
		let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;

		// The request is already being processed by the host, we are free to do anything
		// else in the worker (we can send multiple concurrent requests too).
		// At some point however we probably want to check the response though,
		// so we can block current thread and wait for it to finish.
		// Note that since the request is being driven by the host, we don't have to wait
		// for the request to have it complete, we will just not read the response.
		let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
		// Let's check the status code before we proceed to reading the response.
		if response.code != 200 {
			log::warn!("Unexpected status code: {}", response.code);
			return Err(http::Error::Unknown)
		}

		// Next we want to fully read the response body and collect it to a vector of bytes.
		// Note that the return object allows you to read the body in chunks as well
		// with a way to control the deadline.
		let body = response.body().collect::<Vec<u8>>();

		// Create a str slice from the body.
		let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
			log::warn!("No UTF8 body");
			http::Error::Unknown
		})?;

		let property = match Self::parse_property(body_str) {
			Some(property) => Ok(property),
			None => {
				log::warn!("Unable to extract price from the response: {:?}", body_str);
				Err(http::Error::Unknown)
			},
		}?;

		// log::warn!("Got property: {:?} cents", price);

		Ok(property)
	}

	/// Parse the price from the given JSON string using `lite-json`.
	///
	/// Returns `None` when parsing failed or `Some(price in cents)` when parsing is successful.
	pub fn parse_property(property_str: &str) -> Option<PropertyInfoData<T>> {
		let val = lite_json::parse_json(property_str);
		let id = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'id' field in the first object
						if let Some((_, v)) =
							obj.into_iter().find(|(k, _)| k.iter().copied().eq("id".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::Number(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};
		let val = lite_json::parse_json(property_str);
		let bedrooms = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'bedrooms' field in the first object
						if let Some((_, v)) =
							obj.into_iter().find(|(k, _)| k.iter().copied().eq("bedrooms".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::Number(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};
		let val = lite_json::parse_json(property_str);
		let bathrooms = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'bathrooms' field in the first object
						if let Some((_, v)) =
							obj.into_iter().find(|(k, _)| k.iter().copied().eq("bathrooms".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::Number(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let val = lite_json::parse_json(property_str);
		let summary = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'summary' field in the first object
						if let Some((_, v)) =
							obj.into_iter().find(|(k, _)| k.iter().copied().eq("summary".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::String(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let val = lite_json::parse_json(property_str);
		let property_sub_type = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'propertySubType' field in the first object
						if let Some((_, v)) = obj
							.into_iter()
							.find(|(k, _)| k.iter().copied().eq("propertySubType".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::String(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let val = lite_json::parse_json(property_str);
		let first_visible_date = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'firstVisibleDate' field in the first object
						if let Some((_, v)) = obj
							.into_iter()
							.find(|(k, _)| k.iter().copied().eq("firstVisibleDate".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::String(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let val = lite_json::parse_json(property_str);
		let display_size = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'displaySize' field in the first object
						if let Some((_, v)) = obj
							.into_iter()
							.find(|(k, _)| k.iter().copied().eq("displaySize".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::String(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let val = lite_json::parse_json(property_str);
		let display_address = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'displayAddress' field in the first object
						if let Some((_, v)) = obj
							.into_iter()
							.find(|(k, _)| k.iter().copied().eq("displayAddress".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::String(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let val = lite_json::parse_json(property_str);
		let property_images = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'propertyImages' field in the first object
						if let Some((_, v)) = obj
							.into_iter()
							.find(|(k, _)| k.iter().copied().eq("propertyImages".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::String(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let id = id.integer as u32;
		let bedrooms = bedrooms.integer as u32;
		let bathrooms = bathrooms.integer as u32;
		let summary: &str = &summary.iter().collect::<String>();
		let property_sub_type: &str = &property_sub_type.iter().collect::<String>();
		let first_visible_date: &str = &first_visible_date.iter().collect::<String>();
		let display_size: &str = &display_size.iter().collect::<String>();
		let display_address: &str = &display_address.iter().collect::<String>();
		let property_images: &str = &property_images.iter().collect::<String>();

		let property = PropertyInfoData {
			id,
			bedrooms,
			bathrooms,
			summary: summary.as_bytes().to_vec().try_into().unwrap(),
			property_sub_type: property_sub_type.as_bytes().to_vec().try_into().unwrap(),
			first_visible_date: first_visible_date.as_bytes().to_vec().try_into().unwrap(),
			display_size: display_size.as_bytes().to_vec().try_into().unwrap(),
			display_address: display_address.as_bytes().to_vec().try_into().unwrap(),
			property_images1: property_images.as_bytes().to_vec().try_into().unwrap(),
		};

		Some(property)

		// Some(price.integer as u32 * 100 + (price.fraction / 10_u64.pow(exp)) as u32)
	}

	/* pub fn fetch_property_and_send_signed() -> DispatchResult {
		let signer = Signer::<T, T::AuthorityId>::all_accounts();
		if !signer.can_sign() {
			return Err(
				"No local accounts available. Consider adding one via `author_insertKey` RPC.",
			)
		}
		// Make an external HTTP request to fetch the current price.
		// Note this call will block until response is received.
		let property = Self::fetch_property().map_err(|_| "Failed to fetch price")?;

		// Using `send_signed_transaction` associated type we create and submit a transaction
		// representing the call, we've just created.
		// Submit signed will return a vector of results for all accounts that were found in the
		// local keystore with expected `KEY_TYPE`.
		let results = signer.send_signed_transaction(|_account| {
			// Received price is wrapped into a call to `submit_price` public function of this
			// pallet. This means that the transaction, when executed, will simply call that
			// function passing `price` as an argument.
			//Call::submit_price { property: property.clone() }
		});

		for (acc, res) in &results {
			match res {
				Ok(()) => log::info!("[{:?}] Submitted price of {:?} cents", acc.id, property),
				Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
			}
		}

		Ok(())
	} */
}
