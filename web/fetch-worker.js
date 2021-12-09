import init from "mapr";

onmessage = async m => {
    let msg = m.data;

    await fetch("http://localhost:8080/mapr.html")

    if (msg.type === "init") {
        const init_output = await init(undefined, msg.memory);
        console.log(msg.address)
        postMessage(init_output.get54(msg.address));
    }
};