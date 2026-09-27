import { operatorStart } from "./operator-client.mjs";
const result = await operatorStart({ privateRoot: process.argv[2] });
process.stdout.write(JSON.stringify(result) + "\n");
