/* ea:   0x0003E050 */
/* name: sub_3E050 */
/* size: 84 */
int __fastcall sub_3E050(int a1, _BYTE *a2)
{
  _BYTE *v4; // r0

  *(_DWORD *)a1 = 0;
  v4 = sub_181F4(a1, 1);
  *(_DWORD *)(a1 + 4) = 0;
  *(_DWORD *)a1 = v4;
  *(_BYTE *)(a1 + 8) = *a2;
  *v4 = 0;
  *(_DWORD *)(*(_DWORD *)a1 + 4) = 0;
  *(_DWORD *)(*(_DWORD *)a1 + 8) = *(_DWORD *)a1;
  *(_DWORD *)(*(_DWORD *)a1 + 12) = *(_DWORD *)a1;
  return a1;
}

