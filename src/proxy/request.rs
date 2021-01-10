use std::str;
use regex;

#[derive(Debug)]
pub struct Info
{
	pub method: Option<String>,
	pub protocol: Option<String>,
	pub host: Option<String>,
	pub port: Option<String>,
	pub uri: Option<String>,
	pub version: Option<String>,
}

impl Info
{
	pub fn new(raw_request: &[u8]) -> Info
	{
		let mut eol_index = 0;
		for ch in raw_request
		{
			eol_index += 1;
			if *ch == 10u8
			{
				break;
			}
		}

		let string_re = regex::Regex::new(r"^(\S+) (?:(https?|tcp|ftp|ssh)://)?([\-\.a-zA-Z0-9]+)(?::(\d{1,5}))?((?:/[\-\.a-zA-Z0-9\?\&=]*)*)? (\S+)").unwrap();

		let mut request_info = Info
		{
			method: None,
			protocol: None,
			host: None,
			port: None,
			uri: None,
			version: None,
		};

		if let Some(re_search_results) = string_re.captures(str::from_utf8(&raw_request[0 .. eol_index]).unwrap())
		{
			if let Some(group) = re_search_results.get(1)
			{
				request_info.method = Some(String::from(group.as_str()));
			}
			if let Some(group) = re_search_results.get(2)
			{
				request_info.protocol = Some(String::from(group.as_str()));
			}
			if let Some(group) = re_search_results.get(3)
			{
				request_info.host = Some(String::from(group.as_str()));
			}
			if let Some(group) = re_search_results.get(4)
			{
				request_info.port = Some(String::from(group.as_str()));
			}
			else
			{
				match request_info.protocol.clone().unwrap().as_str()
				{
					p if p == "https" =>
					{
						request_info.port = Some(String::from("443"));
					},
					p if p == "ftp" =>
					{
						request_info.port = Some(String::from("21"));
					},
					p if p == "ssh" =>
					{
						request_info.port = Some(String::from("22"));
					},
					_ =>
					{
						request_info.port = Some(String::from("80"));
					},
				}
			}
			if let Some(group) = re_search_results.get(5)
			{
				request_info.uri = Some(String::from(group.as_str()));
			}
			if let Some(group) = re_search_results.get(6)
			{
				request_info.version = Some(String::from(group.as_str()));
			}
		}

		return request_info;
	}
}

pub fn from_bytes(raw_request: &[u8]) -> Vec<u8>
{
	let request_info = Info::new(raw_request);
	let mut formatted_request = Vec::new();
	let mut first_request_line = String::new();

	first_request_line += &request_info.method.unwrap();
	first_request_line += " ";
	first_request_line += &request_info.uri.unwrap();
	first_request_line += " ";
	first_request_line += &request_info.version.unwrap();

	let first_line_eol = raw_request.iter().position(|ch| { return *ch == 13u8; }).unwrap();

	formatted_request.extend_from_slice(first_request_line.as_bytes());
	formatted_request.extend_from_slice(&raw_request[first_line_eol ..]);

	return formatted_request;
}

