import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button, Card, IconButton, Stack } from "@mui/material";
import DeleteIcon from '@mui/icons-material/Delete';
import AddIcon from '@mui/icons-material/Add';
import { useNavigate } from "react-router-dom";

function StockList() {
    const [stocks, setStocks] = useState<string[]>([]);

    useEffect(() => {
        invoke("get_portfolio").then((result) => {
            setStocks(result as string[]);
        }).catch((error) => {
            console.error(error);
        });
    }, []);

    function addStock(stock: string) {
        const added = [...stocks, stock];
        invoke("set_portfolio", { portfolio: added }).then(() => {
            setStocks(added);
        }).catch((error) => {
            console.error(error);
        });
    }

    function removeStock(index: number) {
        const updatedStocks = stocks.filter((_, i) => i !== index);
        invoke("set_portfolio", { portfolio: updatedStocks }).then(() => {
            setStocks(updatedStocks);
        }).catch((error) => {
            console.error(error);
        });
    }

    return (
        <>
            <h1>Stock List</h1>
            <Stack>
                {
                    stocks.map((stock, index) => (
                        <StockCard key={index} code={stock} index={index} deleteStock={removeStock} />
                    ))
                }
                <AddStockButton addStock={addStock} />
            </Stack>
        </>
    )
}

function AddStockButton({ addStock }: { addStock: (stock: string) => void }) {
    return (
        <div style={{ display: "flex", justifyContent: "center" }}>
            <IconButton
                style={{ color: "green" }}
                onClick={() => {
                    const stockCode = prompt("Enter stock code");
                    if (stockCode) {
                        addStock(stockCode);
                    }
                }}
            >
                <AddIcon />
            </IconButton>
        </div>
    );
}

function StockCard({ code, index, deleteStock }: { code: string, index: number, deleteStock: (index: number) => void }) {
    const navigate = useNavigate();
    return (
        <Card variant="outlined" style={{ margin: "10px", padding: "20px" }}>
            <h3>{code}</h3>
            <div style={{ display: "flex", justifyContent: "space-between" }}>
                <Button
                    variant="contained"
                    color="primary"
                    onClick={() => {
                        navigate(
                            "/second",
                            { state: { code: code } }
                        )
                    }}
                >
                    View Chart
                </Button>
                <IconButton onClick={() => deleteStock(index)}>
                    <DeleteIcon />
                </IconButton>
            </div>
        </Card>
    );

}

export default StockList;