/* ea:   0x00061B74 */
/* name: sub_61B74 */
/* size: 488 */
unsigned __int16 *__fastcall sub_61B74(unsigned __int16 *a1, int a2, unsigned __int16 a3)
{
  unsigned __int16 *v3; // r5
  int v5; // r3
  char v6; // r4
  unsigned __int16 v7; // t1
  unsigned __int16 v8; // t1
  unsigned __int16 v9; // t1
  int v10; // r1
  unsigned __int16 *result; // r0

  v3 = a1;
  v5 = (a2 - (int)a1) >> 3;
  v6 = 1;
  if ( v5 > 0 )
  {
    while ( a3 == *v3 )
    {
      v7 = v3[1];
      ++v3;
      if ( a3 != v7 )
        break;
      v8 = v3[1];
      ++v3;
      if ( a3 != v8 )
        break;
      v9 = v3[1];
      ++v3;
      if ( a3 != v9 )
        break;
      --v5;
      ++v3;
      if ( v5 <= 0 )
        goto LABEL_7;
    }
    return v3;
  }
LABEL_7:
  v10 = (a2 - (int)v3) >> 1;
  if ( v10 != 1 )
  {
    if ( v10 != 2 )
    {
      if ( v10 != 3 )
        return (unsigned __int16 *)a2;
      if ( *v3 != a3 )
        return v3;
      ++v3;
    }
    if ( *v3 == a3 )
    {
      ++v3;
      goto LABEL_14;
    }
    return v3;
  }
LABEL_14:
  if ( *v3 == a3 )
    v6 = 0;
  result = v3;
  if ( v6 == 0 )
    return (unsigned __int16 *)a2;
  return result;
}

