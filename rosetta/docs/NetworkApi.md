# \NetworkApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**network_list**](NetworkApi.md#network_list) | **POST** /network/list | Get List of Available Networks
[**network_options**](NetworkApi.md#network_options) | **POST** /network/options | Get Network Options
[**network_status**](NetworkApi.md#network_status) | **POST** /network/status | Get Network Status



## network_list

> crate::models::NetworkListResponse network_list(metadata_request)
Get List of Available Networks

This endpoint returns a list of NetworkIdentifiers that the Rosetta server supports. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**metadata_request** | [**MetadataRequest**](MetadataRequest.md) |  | [required] |

### Return type

[**crate::models::NetworkListResponse**](NetworkListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## network_options

> crate::models::NetworkOptionsResponse network_options(network_request)
Get Network Options

This endpoint returns the version information and allowed network-specific types for a NetworkIdentifier. Any NetworkIdentifier returned by /network/list should be accessible here.  Because options are retrievable in the context of a NetworkIdentifier, it is possible to define unique options for each network. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**network_request** | [**NetworkRequest**](NetworkRequest.md) |  | [required] |

### Return type

[**crate::models::NetworkOptionsResponse**](NetworkOptionsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## network_status

> crate::models::NetworkStatusResponse network_status(network_request)
Get Network Status

This endpoint returns the current status of the network requested. Any NetworkIdentifier returned by /network/list should be accessible here. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**network_request** | [**NetworkRequest**](NetworkRequest.md) |  | [required] |

### Return type

[**crate::models::NetworkStatusResponse**](NetworkStatusResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

