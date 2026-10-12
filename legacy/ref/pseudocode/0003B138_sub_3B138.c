/* ea:   0x0003B138 */
/* name: sub_3B138 */
/* size: 368 */
unsigned __int16 *__fastcall sub_3B138(
        unsigned __int16 *a1,
        unsigned __int16 *a2,
        unsigned __int16 *a3,
        unsigned __int16 *a4)
{
  unsigned __int16 *v4; // r11
  int v7; // r2
  int v9; // r2
  unsigned __int16 *v10; // r5
  unsigned __int16 *v11; // r3
  unsigned __int16 *v12; // r4

  v4 = a1;
  if ( a1 == a2 || a3 == a4 )
    return v4;
  if ( a3 + 1 == a4 )
  {
    v7 = *a3;
    do
    {
      if ( v7 == *v4 )
        break;
      ++v4;
    }
    while ( v4 != a2 );
    return v4;
  }
  do
  {
    v9 = *a3;
    while ( v9 != *v4 )
    {
      if ( ++v4 == a2 )
        return a2;
    }
    if ( v4 == a2 )
      return a2;
    while ( v9 != *v4 )
    {
      if ( ++v4 == a2 )
        return a2;
    }
    if ( v4 == a2 )
      return a2;
    v10 = v4 + 1;
    v11 = v4 + 1;
    v12 = a3 + 1;
    if ( v4 + 1 == a2 )
      return a2;
    while ( *v11 == *v12 )
    {
      if ( ++v12 == a4 )
        return v4;
      if ( ++v11 == a2 )
        return a2;
    }
    ++v4;
  }
  while ( v10 != a2 );
  return v10;
}

