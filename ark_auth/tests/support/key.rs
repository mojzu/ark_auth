#[macro_export]
macro_rules! key_integration_test {
    () => {
        #[test]
        #[ignore]
        fn api_key_list_forbidden() {
            let mut client = client_create();

            client.options.set_authorisation(INVALID_SERVICE_KEY);
            let res = client
                .key_list(KeyListQuery {
                    gt: None,
                    lt: None,
                    limit: None,
                })
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::Forbidden));
        }

        #[test]
        #[ignore]
        fn api_key_list_bad_request_invalid_gt() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);

            client.options.set_authorisation(&service_key.value);
            let res = client
                .key_list(KeyListQuery {
                    gt: Some("".to_owned()),
                    lt: None,
                    limit: None,
                })
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_key_list_bad_request_invalid_lt() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);

            client.options.set_authorisation(&service_key.value);
            let res = client
                .key_list(KeyListQuery {
                    gt: None,
                    lt: Some("".to_owned()),
                    limit: None,
                })
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_key_list_bad_request_invalid_limit() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);

            client.options.set_authorisation(&service_key.value);
            let res = client
                .key_list(KeyListQuery {
                    gt: None,
                    lt: None,
                    limit: Some("-1".to_owned()),
                })
                .unwrap_err();
            assert_eq!(res, Error::Request(RequestError::BadRequest));
        }

        #[test]
        #[ignore]
        fn api_key_list_ok() {
            let mut client = client_create();
            let (_service, service_key) = service_key_create(&client);
            let user_email = email_create();

            client.options.set_authorisation(&service_key.value);
            let user = user_create(&client, true, USER_NAME, &user_email, None);
            let limit = "3";

            client
                .key_create(true, KEY_NAME, None, Some(user.id.to_owned()))
                .unwrap();
            client
                .key_create(true, KEY_NAME, None, Some(user.id.to_owned()))
                .unwrap();
            client
                .key_create(true, KEY_NAME, None, Some(user.id.to_owned()))
                .unwrap();
            client
                .key_create(true, KEY_NAME, None, Some(user.id.to_owned()))
                .unwrap();
            client
                .key_create(true, KEY_NAME, None, Some(user.id.to_owned()))
                .unwrap();

            let res1 = client
                .key_list(KeyListQuery {
                    gt: None,
                    lt: None,
                    limit: Some(limit.to_owned()),
                })
                .unwrap();
            assert_eq!(res1.data.len(), 3);
            let r1_1 = &res1.data[0];
            let r1_2 = &res1.data[1];
            let r1_3 = &res1.data[2];

            let res2 = client
                .key_list(KeyListQuery {
                    gt: Some(r1_1.to_owned()),
                    lt: None,
                    limit: Some(limit.to_owned()),
                })
                .unwrap();
            assert_eq!(res2.data.len(), 3);
            let r2_2 = &res2.data[0];
            let r2_3 = &res2.data[1];
            let r2_4 = &res2.data[2];
            assert_eq!(r2_2, r1_2);
            assert_eq!(r2_3, r1_3);

            let res3 = client
                .key_list(KeyListQuery {
                    gt: Some(r1_2.to_owned()),
                    lt: None,
                    limit: Some(limit.to_owned()),
                })
                .unwrap();
            assert_eq!(res3.data.len(), 3);
            let r3_3 = &res3.data[0];
            let r3_4 = &res3.data[1];
            let r3_5 = &res3.data[2];
            assert_eq!(r3_3, r2_3);
            assert_eq!(r3_4, r2_4);

            let res4 = client
                .key_list(KeyListQuery {
                    gt: None,
                    lt: Some(r3_5.to_owned()),
                    limit: Some(limit.to_owned()),
                })
                .unwrap();
            assert_eq!(res4.data.len(), 3);
            let r4_2 = &res4.data[0];
            let r4_3 = &res4.data[1];
            let r4_4 = &res4.data[2];
            assert_eq!(r4_2, r2_2);
            assert_eq!(r4_3, r3_3);
            assert_eq!(r4_4, r3_4);

            let res5 = client
                .key_list(KeyListQuery {
                    gt: None,
                    lt: Some(r4_4.to_owned()),
                    limit: Some(limit.to_owned()),
                })
                .unwrap();
            assert_eq!(res5.data.len(), 3);
            let r5_1 = &res5.data[0];
            let r5_2 = &res5.data[1];
            let r5_3 = &res5.data[2];
            assert_eq!(r5_1, r1_1);
            assert_eq!(r5_2, r4_2);
            assert_eq!(r5_3, r4_3);
        }

        #[test]
        #[ignore]
        fn api_key_create_forbidden() {
            let mut client = client_create();

            client.options.set_authorisation(INVALID_SERVICE_KEY);
            let res = client.key_create(true, KEY_NAME, None, None).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::Forbidden));
        }

        #[test]
        #[ignore]
        fn api_key_read_forbidden() {
            let mut client = client_create();

            client.options.set_authorisation(INVALID_SERVICE_KEY);
            let res = client.key_read(INVALID_UUID).unwrap_err();
            assert_eq!(res, Error::Request(RequestError::Forbidden));
        }
    };
}