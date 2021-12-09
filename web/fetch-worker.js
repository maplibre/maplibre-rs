import init from "./mapr.js";

var init_output;

onmessage = async m => {
    let msg = m.data;

    if (msg.type === "init") {
        init_output = await init(undefined, msg.memory);
        console.log(msg.address)
        postMessage(init_output.get54(msg.address));
    }
};