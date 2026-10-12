/* ea:   0x00016AA0 */
/* name: sub_16AA0 */
/* size: 56 */
int __fastcall sub_16AA0(int a1, char a2)
{
  *(_BYTE *)(a1 + 4) = a2;
  sub_1333C((_DWORD *)(a1 + 8));
  sub_1333C((_DWORD *)(a1 + 20));
  sub_1333C((_DWORD *)(a1 + 36));
  *(_DWORD *)a1 = &off_6C904;
  return a1;
}

