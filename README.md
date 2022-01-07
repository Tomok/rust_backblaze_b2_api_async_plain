# backblaze b2 api wrapper


** THIS IS CURRENTLY Work in Progress **



[![Rust](https://github.com/Tomok/rust_backblaze_b2_api_async_plain/actions/workflows/rust.yml/badge.svg)](https://github.com/Tomok/rust_backblaze_b2_api_async_plain/actions/workflows/rust.yml)

This is a very thin wrapper around [backblazes b2 api](https://www.backblaze.com/b2/docs/).
Some highlights:
* individual return types to prevent errors at compile time (for example an upload url is it's own type so that the compiler ensures that you can only pass an upload url recevied via the right methods to calls needing an upload url instead of an api url)
* individual error types so that compile time checks can be used to ensure you have captured all known error cases
* [serde](https://serde.rs) support

## Progress
| function  | implemented  | in all calls example  |
|---|---|---|
|[b2_authorize_account](https://www.backblaze.com/b2/docs/b2_authorize_account.html)|✔️|✔️|
|[b2_cancel_large_file](https://www.backblaze.com/b2/docs/b2_cancel_large_file.html)|✔️|✔️|
|[b2_copy_file](https://www.backblaze.com/b2/docs/b2_copy_file.html)|✔️²³|✔️|
|[b2_copy_part](https://www.backblaze.com/b2/docs/b2_copy_part.html)|✔️²|✔️|
|[b2_create_bucket](https://www.backblaze.com/b2/docs/b2_create_bucket.html)|✔️³|✔️|
|[b2_create_key](https://www.backblaze.com/b2/docs/b2_create_key.html)|✔️³|✔️|
|[b2_delete_bucket](https://www.backblaze.com/b2/docs/b2_delete_bucket.html)|✔️³|✔️|
|[b2_delete_file_version](https://www.backblaze.com/b2/docs/b2_delete_file_version.html)|✔️|✔️|
|[b2_delete_key](https://www.backblaze.com/b2/docs/b2_delete_key.html)|✔️³|✔️|
|[b2_download_file_by_id](https://www.backblaze.com/b2/docs/b2_download_file_by_id.html)|✔️|✔️|
|[b2_download_file_by_name](https://www.backblaze.com/b2/docs/b2_download_file_by_name.html)|✔️¹²|✔️|
|[b2_finish_large_file](https://www.backblaze.com/b2/docs/b2_finish_large_file.html)|✔️|✔️|
|[b2_get_download_authorization](https://www.backblaze.com/b2/docs/b2_get_download_authorization.html)|✔️|✔️|
|[b2_get_file_info](https://www.backblaze.com/b2/docs/b2_get_file_info.html)|✔️|✔️|
|[b2_get_upload_part_url](https://www.backblaze.com/b2/docs/b2_get_upload_part_url.html)|✔️|✔️|
|[b2_get_upload_url](https://www.backblaze.com/b2/docs/b2_get_upload_url.html)|✔️|✔️|
|[b2_hide_file](https://www.backblaze.com/b2/docs/b2_hide_file.html)|✔️|✔️|
|[b2_list_buckets](https://www.backblaze.com/b2/docs/b2_list_buckets.html)|✔️³|✔️|
|[b2_list_file_names](https://www.backblaze.com/b2/docs/b2_list_file_names.html)|✔️|✔️|
|[b2_list_file_versions](https://www.backblaze.com/b2/docs/b2_list_file_versions.html)|✔️|✔️|
|[b2_list_keys](https://www.backblaze.com/b2/docs/b2_list_keys.html)|✔️³|✔️|
|[b2_list_parts](https://www.backblaze.com/b2/docs/b2_list_parts.html)|✔️|✔️|
|[b2_list_unfinished_large_files](https://www.backblaze.com/b2/docs/b2_list_unfinished_large_files.html)|✔️|✔️|
|[b2_start_large_file](https://www.backblaze.com/b2/docs/b2_start_large_file.html)|✔️²³|✔️|
|[b2_update_bucket](https://www.backblaze.com/b2/docs/b2_update_bucket.html)|✔️³|✔️|
|[b2_update_file_legal_hold](https://www.backblaze.com/b2/docs/b2_update_file_legal_hold.html)|✔️|✔️|
|[b2_update_file_retention](https://www.backblaze.com/b2/docs/b2_update_file_retention.html)|✔️|✔️|
|[b2_upload_file](https://www.backblaze.com/b2/docs/b2_upload_file.html)|✔️²|✔️|
|[b2_upload_part](https://www.backblaze.com/b2/docs/b2_upload_part.html)|✔️²|✔️|
 
 ¹ Only a base set of parameters supported

 ² Server Side Encryption missing
 
 ³ some fields supported as serde_json::Value only ... to be changed
 
## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
