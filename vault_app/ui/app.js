document.addEventListener('DOMContentLoaded', () => {
    const btn = document.getElementById('test-btn');
    if (btn) {
        btn.innerText = "Settings Load Teszt";
        btn.addEventListener('click', () => {
            if (window.alakit) {
                window.alakit('settings:load|');
            } else {
                alert("Alakit híd nem elérhető!");
            }
        });
    }

    // Add another button to test the settings save controller
    const container = document.querySelector('.container');
    const btn2 = document.createElement('button');
    btn2.innerText = "Settings Save Teszt";
    btn2.style.marginLeft = "10px";
    btn2.addEventListener('click', () => {
        if (window.alakit) {
            window.alakit('settings:save|theme=dark');
        }
    });
    container.appendChild(btn2);

    // Unknown controller test
    const btn3 = document.createElement('button');
    btn3.innerText = "Unknown Controller";
    btn3.style.marginLeft = "10px";
    btn3.addEventListener('click', () => {
        if (window.alakit) {
            window.alakit('unknown:cmd|');
        }
    });
    container.appendChild(btn3);
});
