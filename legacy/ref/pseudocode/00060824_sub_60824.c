/* ea:   0x00060824 */
/* name: sub_60824 */
/* size: 160 */
int __fastcall sub_60824(int *a1, unsigned __int16 *a2, unsigned int a3, int a4)
{
  int v4; // r8
  unsigned __int16 *v5; // r7
  unsigned __int16 *v6; // r6
  unsigned __int16 *v7; // r4
  unsigned __int16 *v8; // r3

  v4 = *a1;
  v5 = (unsigned __int16 *)a1[1];
  if ( a3 >= ((int)v5 - *a1) >> 1 )
    return -1;
  v6 = (unsigned __int16 *)(v4 + 2 * a3);
  if ( v6 == v5 )
  {
LABEL_8:
    v6 = (unsigned __int16 *)a1[1];
  }
  else
  {
    v7 = &a2[a4];
    while ( 1 )
    {
      v8 = a2;
      if ( a2 != v7 )
        break;
LABEL_7:
      if ( ++v6 == v5 )
        goto LABEL_8;
    }
    while ( *v6 != *v8 )
    {
      if ( ++v8 == v7 )
        goto LABEL_7;
    }
  }
  if ( v6 == v5 )
    return -1;
  else
    return ((int)v6 - v4) >> 1;
}

