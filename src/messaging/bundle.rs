//TODO: make a derive macro for automatically converting e.g. a Spawn(Bundle) into an action on the client.
//DON'T TRY TO AUTOMATICALLY SPAWN ANY `&dyn NetBundle`, IT IS WAY TO CONVOLUTED AND DIFFICULT AND YOU SHOULD PREFER HAVING A SYSTEM FOR EVERY BUNDLE WHICH CAN AUTOMATICALLY DOWNCAST IT
