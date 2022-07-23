use crate::execute::stream::source::{
    DataFrameStream, ReadExecutor, ReadTx, WriteExecutor, WriteTx,
};
use crate::repr::df::DataFrame;
use crate::repr::expr::{CreateTable, Insert, MutateRelationExpr, RelationExpr};
use crate::repr::value::{Row, Value};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};

#[async_trait]
impl<W: WriteTx> WriteExecutor<W> for MutateRelationExpr {
    async fn execute_write(self, source: &W) -> Result<Option<DataFrameStream>> {
        match self {
            MutateRelationExpr::CreateTable(n) => n.execute_write(source).await,
            MutateRelationExpr::Insert(n) => n.execute_write(source).await,
        }
    }
}

#[async_trait]
impl<W: WriteTx> WriteExecutor<W> for CreateTable {
    async fn execute_write(self, source: &W) -> Result<Option<DataFrameStream>> {
        source.create_table(self.table, self.schema).await?;
        Ok(None)
    }
}

#[async_trait]
impl<W: WriteTx> WriteExecutor<W> for Insert {
    async fn execute_write(self, source: &W) -> Result<Option<DataFrameStream>> {
        let mut input = self.input.execute_read(source).await?;
        while let Some(stream_result) = input.next().await {
            let df = stream_result?;
            source.insert(&self.table, df).await?;
        }
        Ok(None)
    }
}
