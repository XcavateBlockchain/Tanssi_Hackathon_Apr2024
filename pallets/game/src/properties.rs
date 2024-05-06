use crate::*;
use frame_support::pallet_prelude::*;

impl<T: Config> Pallet<T> {
 	pub(crate) fn create_test_properties() -> DispatchResult {
		let new_property = PropertyInfoData {
			id: 147229391,
			bedrooms: 2,
			bathrooms: 1,
			summary: "Superb 2 double bedroom ground floor purpose-built apartment with sole use of garden. Directly opposite Hackney Downs Park, within walking distance of Clapton, Hackney Downs & Rectory Rd Stations. Benefitting from; 2 double bedrooms, fitted kitchen/diner, modern shower/WC, separate lounge with di...".as_bytes().to_vec().try_into().unwrap(),
			property_sub_type: "Flat".as_bytes().to_vec().try_into().unwrap(),
			first_visible_date: "2024-04-24T16:39:27Z".as_bytes().to_vec().try_into().unwrap(),
			display_size: "".as_bytes().to_vec().try_into().unwrap(),
			display_address: "St Peters Street, Islington".as_bytes().to_vec().try_into().unwrap(),
			property_images1: "https://media.rightmove.co.uk/dir/crop/10:9-16:9/56k/55489/146480642/55489_2291824_IMG_00_0000_max_476x317.jpeg".as_bytes().to_vec().try_into().unwrap(),
			};
		TestProperties::<T>::try_append(new_property.clone()).map_err(|_| Error::<T>::TooManyTest)?;
		TestPrices::<T>::insert(new_property.id, 220000);
		let new_property = PropertyInfoData {
			id: 146480642,
			bedrooms: 2,
			bathrooms: 1,
			summary: "Exceptional, bright and spacious 915 sq ft period upper maisonette for sale with a balcony and 22'2 rear patio garden, presented in good condition and available chain free.".as_bytes().to_vec().try_into().unwrap(),
			property_sub_type: "Flat".as_bytes().to_vec().try_into().unwrap(),
			first_visible_date: "2024-04-24T16:39:27Z".as_bytes().to_vec().try_into().unwrap(),
			display_size: "".as_bytes().to_vec().try_into().unwrap(),
			display_address: "Aragon Tower, London, SE8".as_bytes().to_vec().try_into().unwrap(),
			property_images1: "https://media.rightmove.co.uk/dir/crop/10:9-16:9/128k/127876/147229391/127876_33052394_IMG_12_0000_max_476x317.gif".as_bytes().to_vec().try_into().unwrap(),
			};
		TestProperties::<T>::try_append(new_property.clone()).map_err(|_| Error::<T>::TooManyTest)?;
		TestPrices::<T>::insert(new_property.id, 650000);
		let new_property = PropertyInfoData {
			id: 147031382,
			bedrooms: 3,
			bathrooms: 2,
			summary: "Exceptional, bright and spacious 915 sq ft period upper maisonette for sale with a balcony and 22'2 rear patio garden, presented in good condition and available chain free.".as_bytes().to_vec().try_into().unwrap(),
			property_sub_type: "Flat".as_bytes().to_vec().try_into().unwrap(),
			first_visible_date: "2024-04-24T16:39:27Z".as_bytes().to_vec().try_into().unwrap(),
			display_size: "".as_bytes().to_vec().try_into().unwrap(),
			display_address: "Aragon Tower, London, SE8".as_bytes().to_vec().try_into().unwrap(),
			property_images1: "https://media.rightmove.co.uk/dir/crop/10:9-16:9/128k/127876/147229391/127876_33052394_IMG_12_0000_max_476x317.gif".as_bytes().to_vec().try_into().unwrap(),
			};
		TestProperties::<T>::try_append(new_property.clone()).map_err(|_| Error::<T>::TooManyTest)?;
		TestPrices::<T>::insert(new_property.id, 525000);
		let new_property = PropertyInfoData {
			id: 147031382,
			bedrooms: 3,
			bathrooms: 2,
			summary: "Exceptional, bright and spacious 915 sq ft period upper maisonette for sale with a balcony and 22'2 rear patio garden, presented in good condition and available chain free.".as_bytes().to_vec().try_into().unwrap(),
			property_sub_type: "Flat".as_bytes().to_vec().try_into().unwrap(),
			first_visible_date: "2024-04-24T16:39:27Z".as_bytes().to_vec().try_into().unwrap(),
			display_size: "".as_bytes().to_vec().try_into().unwrap(),
			display_address: "Aragon Tower, London, SE8".as_bytes().to_vec().try_into().unwrap(),
			property_images1: "https://media.rightmove.co.uk/dir/crop/10:9-16:9/128k/127876/147229391/127876_33052394_IMG_12_0000_max_476x317.gif".as_bytes().to_vec().try_into().unwrap(),
			};
		TestProperties::<T>::try_append(new_property.clone()).map_err(|_| Error::<T>::TooManyTest)?;
		TestPrices::<T>::insert(new_property.id, 525000);
		Ok(())
	}  
}
