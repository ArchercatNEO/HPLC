export function ProcessFile(file, noise) {


const reader = new FileReader();
$area = 0;
$baseline = [0];
$floor = [];
$peaks = [];
$gapped = [];

//parse the file
const raw = reader.readAsText(file, "UTF-8");
reader.onload = process;
const byRow = explode("\n", raw);


/*************************************************************************************************
	esto lee el archivo y saca los datos y los guarda en
	 times[]  y  data[]
*************************************************************************************************/
$ii=0;
foreach($raw as $key=>$value){
	$time = 1 * (explode("\t", $value)[0]);
	if (8.85 > $time || $time > 36) continue;
	
	$times[$ii] = 1 * (explode("\t", $value)[0]);
	$labels[$ii] = $times[$ii] % 150 == 0 ? "$times[$ii]" : "";
	$data[$ii] = 1 * (explode("\t", $value)[1]);
	$ii++;
}

/*************************************************************************************************
	esto calcula el baseline, al terminar 
	 baseline[] tiene los indices donde hay que poner los puntos del baseline
*************************************************************************************************/
while (end($baseline) + 1 < count($data)){
	
	$gradient = 1e50;
	$best = end($baseline);
	// esto es para que no haya puntos consecutivos, 
	// lo forzamos a que sean al menos 10 puntos entre minimos
	$end = end($baseline) + 5;

	for ($index = $end + 1; $index < count($data); $index++)
	{
		$grad_i = ($data[$index] - $data[$end])/($index - $end);
		if ( $grad_i < $gradient)
		{
			$gradient = $grad_i;
			$best = $index;
		}
	}
	

	$baseline[] = $best;
}


/*************************************************************************************************
	esto conecta todos los puntos del baseline interpolando los puntos anteriores
	y los guarda en $floor[]
*************************************************************************************************/
for ($index = 1; $index < count($baseline); $index++){

	$start = $baseline[$index - 1];
	$end = $baseline[$index];

	$m = ($data[$end] - $data[$start])/($end - $start);

//	echo "\n $data[$start]";

	for ($x = 0; $x < $end - $start; $x++)
		$floor[] = $m * $x + $data[$start];
	
}


for ($index = 1; $index < count($data); $index++){

	while ($data[$index - 1] > $data[$index]){
		$area 
		+= abs(($data[$index - 1] - $data[$index])/2) 
		+ $data[$index - 1] - $floor[$index - 1]
		+ abs(($floor[$index - 1] - $floor[$index])/2);
		$index++;
	}
	
	if ($index >= count($data)) break;

	$peaks[] = $index;
	$gapped[$index] = $data[$index];

	while ($data[$index - 1] < $data[$index]){
		$area 
		+= abs(($data[$index - 1] - $data[$index])/2) 
		+ $data[$index - 1] - $floor[$index - 1]
		+ abs(($floor[$index - 1] - $floor[$index])/2);
		$index++;
	}
		
	
	if ($index >= count($data)) break;

	$peaks[] = $index;
	$gapped[$index] = $data[$index];

    }
}
