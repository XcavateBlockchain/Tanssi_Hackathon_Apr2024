use crate::*;
use frame_support::pallet_prelude::*;

impl<T: Config> Pallet<T> {
	pub(crate) fn create_test_properties() -> DispatchResult {
		let new_property = PropertyInfoData {
			id: 1,
			property_type: "Apartment".as_bytes().to_vec().try_into().unwrap(),
			bedrooms: 2,
			bathrooms: 2,
			city: "Drays Yard, Norwich".as_bytes().to_vec().try_into().unwrap(),
			post_code: "GB".as_bytes().to_vec().try_into().unwrap(),
			key_features: "Second floor apartment located a short"
				.as_bytes()
				.to_vec()
				.try_into()
				.unwrap(),
		};
		TestProperties::<T>::try_append(new_property).map_err(|_| Error::<T>::TooManyTest)?;
		TestPrices::<T>::insert(1, 220000);
		let new_property = PropertyInfoData {
			id: 2,
			property_type: "Apartment".as_bytes().to_vec().try_into().unwrap(),
			bedrooms: 4,
			bathrooms: 2,
			city: "Norwich".as_bytes().to_vec().try_into().unwrap(),
			post_code: "GB".as_bytes().to_vec().try_into().unwrap(),
			key_features: "A historic and idiosyncratic Grade II"
				.as_bytes()
				.to_vec()
				.try_into()
				.unwrap(),
		};
		TestProperties::<T>::try_append(new_property).map_err(|_| Error::<T>::TooManyTest)?;
		TestPrices::<T>::insert(2, 650000);
		let new_property = PropertyInfoData {
			id: 3,
			property_type: "Town House".as_bytes().to_vec().try_into().unwrap(),
			bedrooms: 3,
			bathrooms: 2,
			city: "Willow Lane, Norwich NR2".as_bytes().to_vec().try_into().unwrap(),
			post_code: "GB".as_bytes().to_vec().try_into().unwrap(),
			key_features: "A truly rare opportunity to secure"
				.as_bytes()
				.to_vec()
				.try_into()
				.unwrap(),
		};
		TestProperties::<T>::try_append(new_property).map_err(|_| Error::<T>::TooManyTest)?;
		TestPrices::<T>::insert(3, 525000);
		let new_property = PropertyInfoData {
			id: 4,
			property_type: "Apartment".as_bytes().to_vec().try_into().unwrap(),
			bedrooms: 4,
			bathrooms: 4,
			city: "Trafalgar Street, Norwich".as_bytes().to_vec().try_into().unwrap(),
			post_code: "GB".as_bytes().to_vec().try_into().unwrap(),
			key_features: "A HIGHLY IMPRESSIVE BLOCK OF FOUR FLATS"
				.as_bytes()
				.to_vec()
				.try_into()
				.unwrap(),
		};
		TestProperties::<T>::try_append(new_property).map_err(|_| Error::<T>::TooManyTest)?;
		TestPrices::<T>::insert(4, 500000);
		Ok(())
	}
}
