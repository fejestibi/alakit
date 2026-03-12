// Alakit Vault JS Interop and Helpers

// Csak a Clipboard API marad meg, mivel ez böngésző-specifikus és JS-ben egyszerűbb kezelni
window.copyToClipboard = function (text) {
    navigator.clipboard.writeText(text).then(function () {
        if (window.alakit_toast) {
            window.alakit_toast("success", "Password copied to clipboard!");
        }
    }).catch(function (err) {
        console.error('Failed to copy: ', err);
    });
}
