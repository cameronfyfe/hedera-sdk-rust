/*
 * ‌
 * Hedera Rust SDK
 * ​
 * Copyright (C) 2022 - 2023 Hedera Hashgraph, LLC
 * ​
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ‍
 */

use hedera_proto::services;
use hedera_proto::services::file_service_client::FileServiceClient;
use hedera_proto::services::smart_contract_service_client::SmartContractServiceClient;
use tonic::transport::Channel;

use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToSchedulableTransactionDataProtobuf,
    ToTransactionDataProtobuf,
    TransactionData,
    TransactionExecute,
};
use crate::{
    BoxGrpcFuture,
    ContractId,
    Error,
    FileId,
    Transaction,
    ValidateChecksums,
};

/// Undelete a file or smart contract that was deleted by a [`SystemUndeleteTransaction`](crate::SystemUndeleteTransaction).
pub type SystemUndeleteTransaction = Transaction<SystemUndeleteTransactionData>;

/// Undelete a file or smart contract that was deleted by  [`SystemUndeleteTransaction`](crate::SystemUndeleteTransaction).
#[derive(Debug, Clone, Default)]
pub struct SystemUndeleteTransactionData {
    file_id: Option<FileId>,
    contract_id: Option<ContractId>,
}

impl SystemUndeleteTransaction {
    /// Returns the contract ID to undelete.
    #[must_use]
    pub fn get_contract_id(&self) -> Option<ContractId> {
        self.data().contract_id
    }

    /// Sets the contract ID to undelete.
    pub fn contract_id(&mut self, id: impl Into<ContractId>) -> &mut Self {
        let data = self.data_mut();
        data.file_id = None;
        data.contract_id = Some(id.into());
        self
    }

    /// Returns the file ID to undelete.
    #[must_use]
    pub fn get_file_id(&self) -> Option<FileId> {
        self.data().file_id
    }

    /// Sets the file ID to undelete.
    pub fn file_id(&mut self, id: impl Into<FileId>) -> &mut Self {
        let data = self.data_mut();
        data.contract_id = None;
        data.file_id = Some(id.into());
        self
    }
}

impl TransactionData for SystemUndeleteTransactionData {}

impl TransactionExecute for SystemUndeleteTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async move {
            if self.file_id.is_some() {
                FileServiceClient::new(channel).system_undelete(request).await
            } else {
                SmartContractServiceClient::new(channel).system_undelete(request).await
            }
        })
    }
}

impl ValidateChecksums for SystemUndeleteTransactionData {
    fn validate_checksums(&self, ledger_id: &crate::ledger_id::RefLedgerId) -> Result<(), Error> {
        self.contract_id.validate_checksums(ledger_id)?;
        self.file_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for SystemUndeleteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::SystemUndelete(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for SystemUndeleteTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::SystemUndelete(self.to_protobuf())
    }
}

impl From<SystemUndeleteTransactionData> for AnyTransactionData {
    fn from(transaction: SystemUndeleteTransactionData) -> Self {
        Self::SystemUndelete(transaction)
    }
}

impl FromProtobuf<services::SystemUndeleteTransactionBody> for SystemUndeleteTransactionData {
    fn from_protobuf(pb: services::SystemUndeleteTransactionBody) -> crate::Result<Self> {
        use services::system_undelete_transaction_body::Id;
        let (file_id, contract_id) = match pb.id {
            Some(Id::FileId(it)) => (Some(FileId::from_protobuf(it)?), None),
            Some(Id::ContractId(it)) => (None, Some(ContractId::from_protobuf(it)?)),
            None => (None, None),
        };

        Ok(Self { file_id, contract_id })
    }
}

impl ToProtobuf for SystemUndeleteTransactionData {
    type Protobuf = services::SystemUndeleteTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let contract_id = self.contract_id.to_protobuf();
        let file_id = self.file_id.to_protobuf();

        let id = match (contract_id, file_id) {
            (Some(contract_id), _) => {
                Some(services::system_undelete_transaction_body::Id::ContractId(contract_id))
            }

            (_, Some(file_id)) => {
                Some(services::system_undelete_transaction_body::Id::FileId(file_id))
            }

            _ => None,
        };
        services::SystemUndeleteTransactionBody { id }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        unused_private_key,
        TEST_NODE_ACCOUNT_IDS,
        TEST_TX_ID,
    };
    use crate::{
        AnyTransaction,
        ContractId,
        FileId,
        Hbar,
        SystemUndeleteTransaction,
    };

    fn make_transaction_file() -> SystemUndeleteTransaction {
        let mut tx = SystemUndeleteTransaction::new();

        tx.node_account_ids(TEST_NODE_ACCOUNT_IDS)
            .transaction_id(TEST_TX_ID)
            .file_id("0.0.444".parse::<FileId>().unwrap())
            .max_transaction_fee(Hbar::new(2))
            .freeze()
            .unwrap()
            .sign(unused_private_key());
        tx
    }

    fn make_transaction_contract() -> SystemUndeleteTransaction {
        let mut tx = SystemUndeleteTransaction::new();

        tx.node_account_ids(TEST_NODE_ACCOUNT_IDS)
            .transaction_id(TEST_TX_ID)
            .contract_id("0.0.444".parse::<ContractId>().unwrap())
            .max_transaction_fee(Hbar::new(2))
            .freeze()
            .unwrap()
            .sign(unused_private_key());
        tx
    }

    #[test]
    fn serialize_file() {
        let tx = make_transaction_file();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            SystemUndelete(
                SystemUndeleteTransactionBody {
                    id: Some(
                        FileId(
                            FileId {
                                shard_num: 0,
                                realm_num: 0,
                                file_num: 444,
                            },
                        ),
                    ),
                },
            )
        "#]]
        .assert_debug_eq(&tx)
    }

    #[test]
    fn serialize_contract() {
        let tx = make_transaction_contract();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            SystemUndelete(
                SystemUndeleteTransactionBody {
                    id: Some(
                        ContractId(
                            ContractId {
                                shard_num: 0,
                                realm_num: 0,
                                contract: Some(
                                    ContractNum(
                                        444,
                                    ),
                                ),
                            },
                        ),
                    ),
                },
            )
        "#]]
        .assert_debug_eq(&tx)
    }

    #[test]
    fn to_from_bytes_file() {
        let tx = make_transaction_file();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);

        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn to_from_bytes_contract() {
        let tx = make_transaction_contract();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);

        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }
}
