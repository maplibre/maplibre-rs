import init, {child_entry_point} from "./dist/libs/mapr"

onmessage = async message => {
    console.warn(message.data)

    const initialised = init(undefined, message.data[1]).catch(err => {
        // Propagate to main `onerror`:
        setTimeout(() => {
            throw err;
        });
        // Rethrow to keep promise rejected and prevent execution of further commands:
        throw err;
    });

    self.onmessage = async message => {
        console.warn(message.data)

        // This will queue further commands up until the module is fully initialised:
        await initialised;
        child_entry_point(message.data);
    };
}
