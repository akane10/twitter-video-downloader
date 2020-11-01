const RESULT_ELM = document.getElementById("result");
const INPUT_URL = document.getElementById("input-url");
const BTN_GET_LINK = document.getElementById("btn-get-link");

const URL = "http://localhost:8002/api";

async function fetch_data() {
  RESULT_ELM.innerHTML = "";

  console.log(INPUT_URL.value)
  try {
    const { data } = await axios.post(URL, {
      url: INPUT_URL.value,
    });

    if (data.length == 0) {
      RESULT_ELM.innerHTML = `<p class="has-text-centered has-text-danger title">Sorry, no result</p>`;
    } else {
      data.forEach((i) => {
        RESULT_ELM.innerHTML += `<a href="${i.url}" class="button tag is-warning is-large">${i.size}</a>`;
      });
    }

    console.log(data);
  } catch (e) {
    console.log(e);
    RESULT_ELM.innerHTML = `<span class="has-text-centered has-text-danger title">Sorry, something went wrong</span>`;
  }
}

BTN_GET_LINK.addEventListener("click", fetch_data);
