/* ea:   0x0006A10C */
/* name: _initterm */
/* size: 48 */
int (*__fastcall initterm(int (*result)(void), unsigned int a2))(void)
{
  int (**i)(void); // r5

  for ( i = (int (**)(void))result; (unsigned int)i < a2; ++i )
  {
    result = *i;
    if ( *i != nullptr )
      result = (int (*)(void))result();
  }
  return result;
}

