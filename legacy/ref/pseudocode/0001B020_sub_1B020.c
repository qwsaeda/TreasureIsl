/* ea:   0x0001B020 */
/* name: sub_1B020 */
/* size: 20 */
void __fastcall sub_1B020(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

