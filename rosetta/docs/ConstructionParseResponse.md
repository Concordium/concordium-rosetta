# ConstructionParseResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**operations** | [**Vec<crate::models::Operation>**](Operation.md) |  | 
**signers** | Option<**Vec<String>**> | [DEPRECATED by `account_identifier_signers` in `v1.4.4`] All signers (addresses) of a particular transaction. If the transaction is unsigned, it should be empty.  | [optional]
**account_identifier_signers** | Option<[**Vec<crate::models::AccountIdentifier>**](AccountIdentifier.md)> |  | [optional]
**metadata** | Option<[**serde_json::Value**](.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


