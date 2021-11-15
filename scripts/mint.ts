const client = new LCDClient({
  URL: "https://fcd.terra.dev",
  chainID: "columbus-5",
  gasPrices: { uusd: 0.35 },
});

const mk = new MnemonicKey({
  mnemonic: "YOUR KEY"
});

const wallet = client.wallet(mk);

const executeTx = await wallet.createAndSignTx({
  msgs: [
    new MsgExecuteContract(
      wallet.key.accAddress,
      "terra13jxycsgusne8rgzp4r2ua3n3qg0l5cufcrnxrl",
      {
        mint_cat: {}
      },
      // the coin to send
      { uluna: 100000}
    ),
  ],
});

const txResult = await client.tx.broadcast(executeTx);
