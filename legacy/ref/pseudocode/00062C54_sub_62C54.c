/* ea:   0x00062C54 */
/* name: sub_62C54 */
/* size: 156 */
int __fastcall sub_62C54(DWORD a1, DWORD a2, HMENU a3, __int16 a4, int a5)
{
  int v9; // r4

  if ( dword_7FBD8 == 0 )
    dword_7FBD8 = 0;
  v9 = a5;
  sub_62F54(&dword_811A8);
  if ( a5 == 0 )
    v9 = 1442840576;
  return sub_62D68(a1, a2, a3, a4, (LPVOID)v9);
}

