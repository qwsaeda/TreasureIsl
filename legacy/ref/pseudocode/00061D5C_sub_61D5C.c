/* ea:   0x00061D5C */
/* name: sub_61D5C */
/* size: 520 */
_DWORD *__fastcall sub_61D5C(_DWORD *result, int a2, int a3, unsigned __int16 a4)
{
  int v4; // r6
  int v6; // r4
  char v7; // r5
  int v8; // r2
  int v9; // r1

  v4 = a2;
  v6 = (a2 - a3) >> 3;
  v7 = 1;
  if ( v6 > 0 )
  {
    while ( a4 == *(unsigned __int16 *)(v4 - 2) )
    {
      v8 = v4 - 2;
      if ( a4 != *(unsigned __int16 *)(v4 - 4)
        || (v8 = v4 - 4, a4 != *(unsigned __int16 *)(v4 - 6))
        || (v8 = v4 - 6, a4 != *(unsigned __int16 *)(v4 - 8)) )
      {
        *result = v8;
        return result;
      }
      --v6;
      v4 -= 8;
      if ( v6 <= 0 )
        goto LABEL_7;
    }
    goto LABEL_17;
  }
LABEL_7:
  v9 = (v4 - a3) >> 1;
  if ( v9 != 1 )
  {
    if ( v9 != 2 )
    {
      if ( v9 != 3 )
        goto LABEL_19;
      if ( *(unsigned __int16 *)(v4 - 2) != a4 )
        goto LABEL_17;
      v4 -= 2;
    }
    if ( *(unsigned __int16 *)(v4 - 2) == a4 )
    {
      v4 -= 2;
      goto LABEL_14;
    }
LABEL_17:
    *result = v4;
    return result;
  }
LABEL_14:
  if ( *(unsigned __int16 *)(v4 - 2) == a4 )
    v7 = 0;
  if ( v7 != 0 )
    goto LABEL_17;
LABEL_19:
  *result = a3;
  return result;
}

