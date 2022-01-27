# \MempoolApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**mempool**](MempoolApi.md#mempool) | **POST** /mempool | Get All Mempool Transactions
[**mempool_transaction**](MempoolApi.md#mempool_transaction) | **POST** /mempool/transaction | Get a Mempool Transaction



## mempool

> crate::models::MempoolResponse mempool(network_request)
Get All Mempool Transactions

Get all Transaction Identifiers in the mempool

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**network_request** | [**NetworkRequest**](NetworkRequest.md) |  | [required] |

### Return type

[**crate::models::MempoolResponse**](MempoolResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## mempool_transaction

> crate::models::MempoolTransactionResponse mempool_transaction(mempool_transaction_request)
Get a Mempool Transaction

Get a transaction in the mempool by its Transaction Identifier. This is a separate request than fetching a block transaction (/block/transaction) because some blockchain nodes need to know that a transaction query is for something in the mempool instead of a transaction in a block.  Transactions may not be fully parsable until they are in a block (ex: may not be possible to determine the fee to pay before a transaction is executed). On this endpoint, it is ok that returned transactions are only estimates of what may actually be included in a block. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**mempool_transaction_request** | [**MempoolTransactionRequest**](MempoolTransactionRequest.md) |  | [required] |

### Return type

[**crate::models::MempoolTransactionResponse**](MempoolTransactionResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

