/* ea:   0x00052DB4 */
/* name: sub_52DB4 */
/* size: 156 */
int __fastcall sub_52DB4(int a1, unsigned __int16 *a2, unsigned int a3)
{
  unsigned int v3; // r6
  unsigned __int16 *v4; // r4
  int v5; // r9

  v3 = 0;
  if ( a3 == 0 )
    return 0;
  v4 = a2;
  v5 = 1;
  while ( *(unsigned __int16 *)((char *)v4 + a1 - (_DWORD)a2) == *v4 )
  {
    ++v3;
    ++v4;
    if ( v3 >= a3 )
      return 0;
  }
  if ( *(unsigned __int16 *)(a1 + 2 * v3) < (unsigned int)a2[v3] )
    return -1;
  return v5;
}

