/* ea:   0x00019DE4 */
/* name: sub_19DE4 */
/* size: 216 */
_BYTE *__fastcall sub_19DE4(_BYTE *a1, int a2, unsigned __int8 a3)
{
  _BYTE *v3; // r4
  int v6; // r2
  int v7; // t1
  int v8; // t1
  int v9; // t1
  int v10; // r1
  _BYTE *result; // r0

  v3 = a1;
  v6 = (a2 - (int)a1) >> 5;
  if ( v6 > 0 )
  {
    while ( a3 != (unsigned __int8)*v3 )
    {
      v7 = (unsigned __int8)v3[8];
      v3 += 8;
      if ( a3 == v7 )
        break;
      v8 = (unsigned __int8)v3[8];
      v3 += 8;
      if ( a3 == v8 )
        break;
      v9 = (unsigned __int8)v3[8];
      v3 += 8;
      if ( a3 == v9 )
        break;
      --v6;
      v3 += 8;
      if ( v6 <= 0 )
        goto LABEL_7;
    }
    return v3;
  }
LABEL_7:
  v10 = (a2 - (int)v3) >> 3;
  switch ( v10 )
  {
    case 1:
LABEL_14:
      result = v3;
      if ( a3 == (unsigned __int8)*v3 )
        return result;
      return (_BYTE *)a2;
    case 2:
      goto LABEL_12;
    case 3:
      if ( a3 == (unsigned __int8)*v3 )
        return v3;
      v3 += 8;
LABEL_12:
      if ( a3 != (unsigned __int8)*v3 )
      {
        v3 += 8;
        goto LABEL_14;
      }
      return v3;
    default:
      break;
  }
  return (_BYTE *)a2;
}

