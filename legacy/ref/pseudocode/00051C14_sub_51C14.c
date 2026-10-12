/* ea:   0x00051C14 */
/* name: sub_51C14 */
/* size: 152 */
int __fastcall sub_51C14(_DWORD *a1, int a2)
{
  int v3; // r7
  int v4; // r6
  int v5; // r8
  _DWORD *i; // r4

  v3 = a1[6] + 608;
  v4 = 0;
  v5 = 0;
  if ( (unsigned int)(*(unsigned __int8 *)(a1[3] + 22) << 26) >> 28 == 0 )
    v4 = *(unsigned __int8 *)(a1[5] + 48);
  for ( i = *(_DWORD **)(*(_DWORD *)a2 + 8); i != *(_DWORD **)a2; i = sub_16A30(i) )
    v5 += sub_53BC4(v3, *((unsigned __int8 *)i + 16), v4, *((unsigned __int16 *)i + 9));
  return v5;
}

