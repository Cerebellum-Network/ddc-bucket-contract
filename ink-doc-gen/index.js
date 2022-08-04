const fs = require("fs");
const {join} = require("path");
const log = console.log;

const ABI_PATH = process.env.ABI_PATH;
let DOC_PATH = process.env.DOC_PATH;
let TEMPLATE_PATH = process.env.TEMPLATE_PATH;

const Handlebars = require("handlebars");


async function main() {
    if (!ABI_PATH) {
        throw "Missing envvar ABI_PATH";
    }
    if (!DOC_PATH) {
        DOC_PATH = ABI_PATH + ".md";
    }
    if (!TEMPLATE_PATH) {
        TEMPLATE_PATH = join(__dirname, "template.handlebars");
    }
    log("Reading   ABI_PATH=" + ABI_PATH);
    log("Rendering TEMPLATE_PATH=" + TEMPLATE_PATH);

    const template = fs.readFileSync(TEMPLATE_PATH, {encoding: "utf8"});
    //log(template);
    const render = Handlebars.compile(template);

    const abi = JSON.parse(fs.readFileSync(ABI_PATH));
    //log(abi);

    const rendered = render(abi);
    fs.writeFileSync(DOC_PATH, rendered);
    log("Written   DOC_PATH=" + DOC_PATH);
}

main().then(() => {
}, (err) => {
    log(err);
    process.exit(1);
});
