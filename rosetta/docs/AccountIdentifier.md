# AccountIdentifier

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**address** | **String** | The address may be a cryptographic public key (or some encoding of it) or a provided username.  | 
**sub_account** | Option<[**crate::models::SubAccountIdentifier**](SubAccountIdentifier.md)> |  | [optional]
**metadata** | Option<[**serde_json::Value**](.md)> | Blockchains that utilize a username model (where the address is not a derivative of a cryptographic public key) should specify the public key(s) owned by the address in metadata.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


