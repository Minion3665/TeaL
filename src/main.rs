use tokio;
use sqlx::Error;
mod database;

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

async fn run() -> Result<(), Error>{
    let mut database = database::Database::new().await?;

    database.setup().await?;

    database.add_task("This is a task").await?;
    database.add_task("This is another task").await?;

    let tasks = database.list_tasks().await?;

    println!("There are {} tasks", tasks.len());

    for task in tasks {
      println!("Task: {} (ID: {})", task.description, task.id)
    }

    Ok(())
}
