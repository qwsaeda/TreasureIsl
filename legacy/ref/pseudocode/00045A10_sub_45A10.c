/* ea:   0x00045A10 */
/* name: sub_45A10 */
/* size: 40 */
void __fastcall sub_45A10(int a1)
{
  void **v1; // r4

  *(_DWORD *)a1 = &off_6FDC8;
  v1 = (void **)(a1 + 4);
  sub_12F48((void **)(a1 + 16));
  sub_12F48(v1);
}

