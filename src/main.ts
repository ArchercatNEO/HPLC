import './style.css'
import { Process } from './analysis.ts'

const slider = document.getElementById("slider") as HTMLInputElement;
const sliderValue = document.getElementById("sliderValue") as HTMLOutputElement;
const files = document.getElementById("fileInput") as HTMLInputElement;
const noise = document.getElementById("noiseInput") as HTMLInputElement;
const button = document.getElementById("send") as HTMLInputElement;

const reader = new FileReader();
reader.onload = event => Process(event, parseFloat(noise.value));

button.addEventListener("click", Submit)

slider.addEventListener("input", () => {
  const step = parseFloat(noise.value);
  const slide = parseFloat(slider.value);
  sliderValue.textContent = String(step * slide);
});

function Submit() {
  if (!files.files) return;

  for (const file of files.files)
    reader.readAsText(file, "UTF-8")  
}

