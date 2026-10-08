/* ea:   0x00030BD8 */
/* name: sub_30BD8 */
/* size: 84 */
int __fastcall sub_30BD8(int a1, _BYTE *a2)
{
  _BYTE *v4; // r0

  *(_DWORD *)a1 = 0;
  v4 = sub_1EE84(a1, 1);
  *(_DWORD *)(a1 + 4) = 0;
  *(_DWORD *)a1 = v4;
  *(_BYTE *)(a1 + 8) = *a2;
  *v4 = 0;
  *(_DWORD *)(*(_DWORD *)a1 + 4) = 0;
  *(_DWORD *)(*(_DWORD *)a1 + 8) = *(_DWORD *)a1;
  *(_DWORD *)(*(_DWORD *)a1 + 12) = *(_DWORD *)a1;
  return a1;
}

