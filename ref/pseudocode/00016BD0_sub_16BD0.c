/* ea:   0x00016BD0 */
/* name: sub_16BD0 */
/* size: 204 */
unsigned __int8 *__fastcall sub_16BD0(unsigned __int8 *a1, int a2, unsigned __int8 *a3)
{
  unsigned __int8 *v3; // r3
  int v6; // r2
  int v7; // r1
  int v8; // t1
  int v9; // t1
  int v10; // t1
  int v11; // r0
  unsigned __int8 *result; // r0

  v3 = a1;
  v6 = (a2 - (int)a1) >> 2;
  if ( v6 > 0 )
  {
    v7 = *a3;
    while ( *v3 != v7 )
    {
      v8 = *++v3;
      if ( v8 == v7 )
        break;
      v9 = *++v3;
      if ( v9 == v7 )
        break;
      v10 = *++v3;
      if ( v10 == v7 )
        break;
      --v6;
      ++v3;
      if ( v6 <= 0 )
        goto LABEL_8;
    }
    return v3;
  }
LABEL_8:
  v11 = a2 - (_DWORD)v3;
  if ( a2 - (_DWORD)v3 == 1 )
  {
LABEL_15:
    result = v3;
    if ( *v3 == *a3 )
      return result;
    return (unsigned __int8 *)a2;
  }
  if ( v11 == 2 )
    goto LABEL_13;
  if ( v11 == 3 )
  {
    if ( *v3 == *a3 )
      return v3;
    ++v3;
LABEL_13:
    if ( *v3 != *a3 )
    {
      ++v3;
      goto LABEL_15;
    }
    return v3;
  }
  return (unsigned __int8 *)a2;
}

