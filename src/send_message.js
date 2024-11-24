const form = document.forms['send_message'];
const messagebox = document.getElementById('chatroom');
form.addEventListener('submit', (e) => {
    console.log("Sending")
    e.preventDefault();
    message = document.createElement("div")
    message.className = "container"
    message.innerHTML = ("<a class=\"author-name\">' + e.elements['author'].value '</a>: <p class=\"message\">'  + e.elements['message'].value + '</p>")

    messagebox.appendChild(messagebox)
});
