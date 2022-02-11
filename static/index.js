function onLoad() {
    localStorage.setItem(
        "path",
        document.getElementById("path").getAttribute("path")
    );

    let cards = document.getElementsByClassName("removable");

    for (var i = 0; i < cards.length; i++) {
        let name = cards[i].getAttribute("name");

        cards[i].children[0].onclick = (_) => remove(name);
    }
}

function remove(name) {
    let modal = document.getElementById("modal");
    let modalContent = document.getElementById("modal-content");
    modalContent.innerHTML = "";

    let text = document.createTextNode("You really wanna remove " + name);
    modalContent.appendChild(text);

    let yes = document.createElement("button");
    yes.innerHTML = "Yes";
    yes.onclick = (_) => {
        fetch("/remove", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                name: name,
                path: localStorage.getItem("path"),
            }),
        }).then((_) => {
            location.reload();
        });
    };
    modalContent.appendChild(yes);

    let cancel = document.createElement("button");
    cancel.innerHTML = "Cancel";
    cancel.onclick = (_) => {
        modal.style.display = "none";
    };
    modalContent.appendChild(cancel);

    modal.style.display = "block";
}

function newFolder() {
    let modal = document.getElementById("modal");
    let modalContent = document.getElementById("modal-content");
    modalContent.innerHTML = "";

    let text = document.createTextNode("Enter the name");
    modalContent.appendChild(text);

    let input = document.createElement("input");
    input.type = "text";
    modalContent.appendChild(input);

    let ok = document.createElement("button");
    ok.innerHTML = "Ok";
    ok.onclick = (_) => {
        fetch("/create", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                name: input.value,
                path: localStorage.getItem("path"),
            }),
        }).then((_) => {
            location.reload();
        });
    };
    modalContent.appendChild(ok);

    let cancel = document.createElement("button");
    cancel.innerHTML = "Cancel";
    cancel.onclick = (_) => {
        modal.style.display = "none";
    };
    modalContent.appendChild(cancel);

    modal.style.display = "block";
}

function uploadFiles() {
    let input = document.createElement("input");
    input.type = "file";
    input.multiple = true;

    input.onchange = (_) => {
        let data = new FormData();

        data.append("path", localStorage.getItem("path"));

        for (var i = 0; i < input.files.length; i++) {
            data.append("file", input.files[i]);
        }

        let modal = document.getElementById("modal");
        let modalContent = document.getElementById("modal-content");
        modalContent.innerHTML = "";

        let text = document.createTextNode("Loading...");
        modalContent.appendChild(text);

        modal.style.display = "block";

        fetch("/upload", {
            method: "POST",
            body: data,
        }).then((_) => {
            location.reload();
        });
    };

    input.click();
    return false;
}
