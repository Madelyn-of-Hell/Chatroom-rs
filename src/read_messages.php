<?php
$raw_json = file_get_contents($filename="/Users/daddyslime/RustroverProjects/non_blocking_input_test/src/message_log.json");
$decoded_json = json_decode($raw_json, true);
foreach ($decoded_json["messages"] as $i) {
    echo '<div class="container">';
    print_r('<a class="author-name">' . $i["author"] . '</a>: <p class="message">'  . $i["message"] . '</p>' );
    echo "</div>";
}
?>