/**
 * 
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { RequestFile } from './models';
import { ResponseAuditManyData } from './responseAuditManyData';

export class ResponseAuditMany {
    'data': Array<ResponseAuditManyData>;

    static discriminator: string | undefined = undefined;

    static attributeTypeMap: Array<{name: string, baseName: string, type: string}> = [
        {
            "name": "data",
            "baseName": "data",
            "type": "Array<ResponseAuditManyData>"
        }    ];

    static getAttributeTypeMap() {
        return ResponseAuditMany.attributeTypeMap;
    }
}
