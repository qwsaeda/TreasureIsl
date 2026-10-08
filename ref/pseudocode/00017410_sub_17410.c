/* ea:   0x00017410 */
/* name: sub_17410 */
/* size: 244 */
_DWORD *__fastcall sub_17410(_DWORD *a1, int a2, unsigned __int8 a3)
{
  _DWORD *v3; // r5
  int v5; // r3
  int v6; // t1
  int v7; // t1
  int v8; // t1
  int v9; // r1
  _DWORD *result; // r0

  v3 = a1;
  v5 = (a2 - (int)a1) >> 4;
  if ( v5 > 0 )
  {
    while ( *(unsigned __int8 *)(*v3 + 4) != a3 )
    {
      v6 = v3[1];
      ++v3;
      if ( *(unsigned __int8 *)(v6 + 4) == a3 )
        break;
      v7 = v3[1];
      ++v3;
      if ( *(unsigned __int8 *)(v7 + 4) == a3 )
        break;
      v8 = v3[1];
      ++v3;
      if ( *(unsigned __int8 *)(v8 + 4) == a3 )
        break;
      --v5;
      ++v3;
      if ( v5 <= 0 )
        goto LABEL_7;
    }
    return v3;
  }
LABEL_7:
  v9 = (a2 - (int)v3) >> 2;
  switch ( v9 )
  {
    case 1:
LABEL_14:
      result = v3;
      if ( *(unsigned __int8 *)(*v3 + 4) == a3 )
        return result;
      return (_DWORD *)a2;
    case 2:
      goto LABEL_12;
    case 3:
      if ( *(unsigned __int8 *)(*v3 + 4) == a3 )
        return v3;
      ++v3;
LABEL_12:
      if ( *(unsigned __int8 *)(*v3 + 4) != a3 )
      {
        ++v3;
        goto LABEL_14;
      }
      return v3;
    default:
      break;
  }
  return (_DWORD *)a2;
}

