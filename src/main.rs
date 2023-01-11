use sqlx::Error;
use tokio;
mod database;
mod ui;
mod sorting;

//%@&@& @&&&&&&&&&&@@@@@@@@@@&@@@&@
//#%&&&&%&&@&@&&&&&&@@&@&@@&&&&@@&&%&&
//%%&&%###&&&&&&@&&&&&&&@&&&&&&&%&@@@@&%%&(
//#%%%#(#*#%&//(%%%%&%&&&&&&&&&&&&&&/%%&@@&&&(&
//#((%#/((#(   *(((#%%##%#%###/#%&&&&%/*(%&%(###&%&
//#(##(/,*(#.  .*/#%%%#%%###/,  .../%%%/**(%&%###%#%&
//##((/*/,((((#((#%%%##%%%#((*///#%%#%## /(&%&((#%%%%
//((//**,(#/(##%%#%%%%#%%#%#%%##(%&%&%#.*/(%%(%/#%%
/////*,.*#(/#(###(###%%%#%#%%%%%&&%&&#,////%##(##
///***,,./%%//*****/%%%%#####%%&&&&%#(,(((//#####
//./**,.*#.,*/*////*.##%%##%%&&&&&%##*.*/###(%%%#(
//**,. ,#   .    ...*#%%%%%#%%#,(%#,.,**%((###%%#(
///(,         *###&%&%%%* ((,,**/#%%%#%&%%##(
//((*,.. ..,*/#%%%%%# *.#///((#%(%%%%&%#%#(,
//*/.,. .. ,*(((#% * ,**/(##/(**#,#%%%%%(
///.*,**,.,*/##*. , ,**/#(,,,*,**(#%%&%
//((/(((**(##%, .#(/**(((*,,*(//(##%%&&%
//(((#(#/(((##%##(//((/(///(((((###%%%&&&/
//((#((#(/(((###%//((/////(/(((####%#%%&&&%
///###((/((/((#%(/*////(((//(//((##%%%%&&&&%
//(###(((/(((#///*/(((####*,,**/(##%%%%&&&&%.
///######(**/*/(#(((#(##(((/////(((##%%&&&&&&
//.,,,***//((((((/((/*****/((((##%%&&&&%
//,,,,,,**/(///*,,,,,,,,,**//(###%%%&%#
//*,,,,,,,*               **/((##%%###
//,,,,,,,*                 /((#####(
//,,****/                 *,*/((##(
//,,****/                  **/((###
//,,*///                  **/((###
//,*///                  **//(#(
//,,,*//                 ,*/((((
//,,,,,*                 ,,*/((.
//****//                 ,,*/((
//***/////                 ,*/(((.
///(//////                 ,*/((((/
/////*/////                 *////(((
/* This is Stanley. Stanley was given to me by /u/jimmybilly100. Stanley is an easter egg to be put
* in this program where possible, because he is such a good boy */

#[tokio::main]
async fn main() {
    run().await.unwrap()
}

async fn run() -> Result<(), Error> {
    let mut db = database::Database::new().await?;

    db.setup().await?;

    db.add_task("This is a task").await?;
    db.add_task("This is another task").await?;

    ui::display_state(
        ui::States::DisplayingTasks(ui::DisplayingTasksStates::Normal),
        &mut db,
    )
    .await?;
    ui::display_state(
        ui::States::DisplayingTasks(ui::DisplayingTasksStates::Create),
        &mut db,
    )
    .await?;
    ui::display_state(
        ui::States::DisplayingTasks(ui::DisplayingTasksStates::Normal),
        &mut db,
    )
    .await?;

    Ok(())
}
