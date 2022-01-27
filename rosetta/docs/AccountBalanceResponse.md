# AccountBalanceResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**block_identifier** | [**crate::models::BlockIdentifier**](BlockIdentifier.md) |  | 
**balances** | [**Vec<crate::models::Amount>**](Amount.md) | A single account may have a balance in multiple currencies.  | 
**metadata** | Option<[**serde_json::Value**](.md)> | Account-based blockchains that utilize a nonce or sequence number should include that number in the metadata. This number could be unique to the identifier or global across the account address.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


