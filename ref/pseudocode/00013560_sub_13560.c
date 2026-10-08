/* ea:   0x00013560 */
/* name: sub_13560 */
/* size: 408 */
_DWORD *__fastcall sub_13560(_DWORD *result, int a2, int a3, unsigned __int16 a4)
{
  int v4; // r6
  int v7; // r4
  char v8; // r5
  int v9; // r2
  int v10; // r3
  int v11; // r1

  v4 = a2;
  v7 = (a2 - a3) >> 3;
  v8 = 1;
  if ( v7 > 0 )
  {
    v9 = a4;
    while ( v9 != *(unsigned __int16 *)(v4 - 2) )
    {
      v10 = v4 - 2;
      if ( v9 == *(unsigned __int16 *)(v4 - 4)
        || (v10 = v4 - 4, v9 == *(unsigned __int16 *)(v4 - 6))
        || (v10 = v4 - 6, v9 == *(unsigned __int16 *)(v4 - 8)) )
      {
        *result = v10;
        return result;
      }
      --v7;
      v4 -= 8;
      if ( v7 <= 0 )
        goto LABEL_8;
    }
    goto LABEL_18;
  }
LABEL_8:
  v11 = (v4 - a3) >> 1;
  if ( v11 != 1 )
  {
    if ( v11 != 2 )
    {
      if ( v11 != 3 )
        goto LABEL_20;
      if ( *(unsigned __int16 *)(v4 - 2) == a4 )
        goto LABEL_18;
      v4 -= 2;
    }
    if ( *(unsigned __int16 *)(v4 - 2) != a4 )
    {
      v4 -= 2;
      goto LABEL_15;
    }
LABEL_18:
    *result = v4;
    return result;
  }
LABEL_15:
  if ( *(unsigned __int16 *)(v4 - 2) != a4 )
    v8 = 0;
  if ( v8 != 0 )
    goto LABEL_18;
LABEL_20:
  *result = a3;
  return result;
}

