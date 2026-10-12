/* ea:   0x0003B8C0 */
/* name: sub_3B8C0 */
/* size: 244 */
_BYTE *__fastcall sub_3B8C0(_BYTE *a1, int a2, unsigned __int8 a3)
{
  _BYTE *v3; // r4
  unsigned int v6; // r0
  int v7; // r2
  int v8; // t1
  int v9; // t1
  int v10; // t1
  int v11; // r1
  _BYTE *result; // r0

  v3 = a1;
  v6 = (int)((unsigned __int64)(954437177LL * (a2 - (int)a1)) >> 32) >> 3;
  v7 = (int)(v6 + (v6 >> 31)) >> 2;
  if ( v7 > 0 )
  {
    while ( a3 != (unsigned __int8)*v3 )
    {
      v8 = (unsigned __int8)v3[36];
      v3 += 36;
      if ( a3 == v8 )
        break;
      v9 = (unsigned __int8)v3[36];
      v3 += 36;
      if ( a3 == v9 )
        break;
      v10 = (unsigned __int8)v3[36];
      v3 += 36;
      if ( a3 == v10 )
        break;
      --v7;
      v3 += 36;
      if ( v7 <= 0 )
        goto LABEL_7;
    }
    return v3;
  }
LABEL_7:
  v11 = (a2 - (int)v3) / 36;
  switch ( v11 )
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
      v3 += 36;
LABEL_12:
      if ( a3 != (unsigned __int8)*v3 )
      {
        v3 += 36;
        goto LABEL_14;
      }
      return v3;
    default:
      break;
  }
  return (_BYTE *)a2;
}

